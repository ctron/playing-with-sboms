use anyhow::anyhow;
use bzip2::read::BzDecoder;
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use spdx_rs::models::SPDX;
use std::fs::DirEntry;
use std::io::{BufReader, Read};
use std::sync::mpsc::sync_channel;
use std::sync::Arc;

fn process_sbom(file: DirEntry) -> anyhow::Result<Option<Arc<SPDX>>> {
    let mut stream = BzDecoder::new(BufReader::new(std::fs::File::open(file.path())?));
    let mut data = vec![];
    stream.read_to_end(&mut data)?;

    log::debug!(
        "Processing {} - on disk: {}, unpacked: {}",
        file.path().display(),
        file.metadata()?.len(),
        data.len()
    );

    match serde_json::from_slice(&data) {
        Err(err) => {
            log::warn!("Failed to parse ({}): {err}", file.path().display());
            Ok(None)
        }
        Ok(spdx) => Ok(Some(Arc::new(spdx))),
    }
}

pub fn walk_sboms<F>(multi: &MultiProgress, mut f: F) -> anyhow::Result<()>
where
    F: FnMut(&ProgressBar, &SPDX) -> anyhow::Result<()>,
{
    let (tx, rx) = sync_channel::<Result<Arc<SPDX>, anyhow::Error>>(10);
    let dir = std::env::current_dir()?.join("data/sboms");

    // scan all candidates

    let mut candidates = vec![];
    for file in std::fs::read_dir(dir)? {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }
        let name = file.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| anyhow!("Unable to convert file name"))?;
        if !name.ends_with(".bz2") {
            continue;
        }

        candidates.push(file);
    }

    let style = ProgressStyle::with_template("{msg:20} {wide_bar} {pos}/{len}")?;

    let len = candidates.len() as _;

    let progress = multi
        .add(ProgressBar::new(len).with_style(style.clone()))
        .with_message("Unpacking SBOMs");

    rayon::spawn(|| {
        let _ = candidates
            .into_par_iter()
            .progress_with(progress)
            .try_for_each_with(tx, |tx, file| match process_sbom(file) {
                Ok(Some(spdx)) => tx.send(Ok(spdx)),
                Ok(None) => Ok(()),
                Err(err) => tx.send(Err(err)),
            });
    });

    let progress = multi.add(
        ProgressBar::new(len)
            .with_style(style)
            .with_message("Processing SBOMs"),
    );

    while let Ok(spdx) = rx.recv() {
        let spdx = spdx?;
        f(&progress, &spdx)?;
        progress.inc(1);
    }

    progress.finish();

    Ok(())
}
