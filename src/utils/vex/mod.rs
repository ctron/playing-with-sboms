pub mod product;

use async_trait::async_trait;
use csaf::Csaf;
use csaf_walker::{
    progress::{Progress, ProgressBar, ProgressImpl},
    retrieve::{RetrievalError, RetrievedAdvisory, RetrievingVisitor},
    source::FileSource,
    walker::Walker,
};
use indicatif::{MultiProgress, ProgressStyle};
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use tokio::sync::Mutex;

pub struct MultiIndicatif(MultiProgress);

impl ProgressImpl for MultiIndicatif {
    fn start(&self, tasks: usize) -> Rc<dyn ProgressBar> {
        let bar = indicatif::ProgressBar::new(tasks.try_into().unwrap_or(u64::MAX));
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {wide_bar} {pos}/{len} ({eta})")
                .unwrap(),
        );
        let bar = self.0.add(bar);
        Rc::new(IndicatifProgressBar(bar))
    }
}

pub struct IndicatifProgressBar(indicatif::ProgressBar);

impl ProgressBar for IndicatifProgressBar {
    fn tick(&self) {
        self.0.inc(1)
    }

    fn set_message(&self, msg: Cow<'static, str>) {
        self.0.set_message(msg);
    }
}

#[async_trait(?Send)]
pub trait VexTask {
    async fn process(&mut self, advisory: ParsedAdvisory) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct ParsedAdvisory {
    pub retrieved: RetrievedAdvisory,
    pub csaf: Csaf,
}

impl Deref for ParsedAdvisory {
    type Target = RetrievedAdvisory;

    fn deref(&self) -> &Self::Target {
        &self.retrieved
    }
}

impl DerefMut for ParsedAdvisory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.retrieved
    }
}

pub async fn run_vex<T>(multi: &MultiProgress, task: &mut T) -> anyhow::Result<()>
where
    T: VexTask,
{
    let progress = Progress::new(MultiIndicatif(multi.clone()));

    let base = std::env::var("CSAF_DATA").unwrap_or_else(|_| "data/vex".to_string());
    log::info!("Loading VEX from: {base}");
    let source = FileSource::new(base).unwrap();

    let task = Mutex::new(task);

    Walker::new(source.clone())
        .with_progress(progress.into())
        .walk(RetrievingVisitor::new(
            source.clone(),
            |retrieved: Result<RetrievedAdvisory, RetrievalError>| async {
                match retrieved {
                    Ok(retrieved) => match serde_json::from_slice(&retrieved.data) {
                        Ok(csaf) => {
                            task.lock()
                                .await
                                .process(ParsedAdvisory { retrieved, csaf })
                                .await?
                        }
                        Err(err) => {
                            log::warn!("Failed to parse document: {err}");
                        }
                    },
                    Err(err) => log::warn!("Failed to retrieve document: {err}"),
                }
                Ok::<_, anyhow::Error>(())
            },
        ))
        .await?;

    Ok(())
}
