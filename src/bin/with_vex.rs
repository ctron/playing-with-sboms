use async_trait::async_trait;
use cpe::{
    component::Component,
    cpe::{Cpe, CpeType, Language},
    uri::Uri,
};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::Level;
use playing_with_sboms::utils::vex::{ParsedAdvisory, VexTask};
use playing_with_sboms::{run::run_task, tasks, utils, utils::vex::run_vex};
use std::collections::BTreeMap;
use std::mem;

#[derive(Default)]
pub struct CollectCpe {
    pub map: BTreeMap<String, usize>,
}

#[async_trait(?Send)]
impl VexTask for CollectCpe {
    async fn process(&mut self, advisory: ParsedAdvisory) -> anyhow::Result<()> {
        let cpes = utils::vex::product::collect_cpe(&advisory.csaf);

        for cpe in cpes {
            *self.map.entry(cpe).or_default() += 1;
        }

        Ok(())
    }
}

impl Drop for CollectCpe {
    fn drop(&mut self) {
        println!("{} unique entries", self.map.len());
        for (k, v) in &self.map {
            println!("{k}: {v}");
        }
    }
}

async fn run(multi: &MultiProgress) -> anyhow::Result<()> {
    let mut vex_cpe = CollectCpe::default();
    let mut sbom_cpe = tasks::main_cpe::MainCpe::default();

    run_vex(multi, &mut vex_cpe).await?;
    run_task(multi, &mut sbom_cpe)?;

    let vex_cpe = mem::replace(&mut vex_cpe.map, Default::default());
    let sbom_cpe = mem::replace(&mut sbom_cpe.map, Default::default());

    let (hits, misses) = resolve_cpes(vex_cpe.into_keys(), sbom_cpe.into_keys());

    println!("Hits: {hits}, Misses: {misses}");

    Ok(())
}

fn fixup_cpe(cpe: String) -> String {
    // fix `*` to "ANY"
    let cpe = cpe.replace('*', "");
    // fix last (language) to "ANY"
    if cpe.ends_with(':') {
        let mut c = cpe.chars();
        c.next_back();
        c.collect()
    } else {
        cpe
    }
}

fn resolve_cpes<F, T>(from: F, to: T) -> (usize, usize)
where
    F: IntoIterator<Item = String>,
    T: IntoIterator<Item = String>,
{
    let from = from.into_iter().map(fixup_cpe).collect::<Vec<_>>();
    let to = to.into_iter().map(fixup_cpe).collect::<Vec<_>>();

    let from = from
        .iter()
        .map(|cpe| (cpe, Uri::parse(cpe)))
        .filter_map(|(cpe, r)| match r {
            Ok(uri) => Some(uri),
            Err(err) => {
                log::warn!("Failed to parse CPE ({cpe}): {err}");
                None
            }
        })
        .collect::<Vec<_>>();
    let to = to
        .iter()
        .map(|cpe| (cpe, Uri::parse(cpe)))
        .filter_map(|(cpe, r)| match r {
            Ok(uri) => Some(uri),
            Err(err) => {
                log::warn!("Failed to parse CPE ({cpe}): {err}");
                None
            }
        })
        .collect::<Vec<_>>();

    let mut hits = 0;
    let mut misses = 0;

    println!(r#"VEX,Num,SBOMs"#);

    for cpe in from {
        let targets = to
            .iter()
            .filter(|c| matches(&cpe, c))
            .map(|cpe| format!("{cpe}"))
            .collect::<Vec<_>>();

        let level = if targets.is_empty() {
            misses += 1;
            Level::Warn
        } else {
            hits += 1;
            Level::Info
        };

        //log::log!(level, "{cpe} => [{}]", targets.join(", "));
        println!(r#""{cpe}",{},"[{}]""#, targets.len(), targets.join(" "));
    }

    (hits, misses)
}

macro_rules! match_ele {
    ($a:expr, $b:expr, $t:ty, $name:ident) => {
        if $a.$name() != <$t>::Any && $a.$name() != $b.$name() {
            return false;
        }
    };
    ($a:expr, $b:expr, $t:ty, *$name:ident) => {
        if $a.$name() != &<$t>::Any && $a.$name() != $b.$name() {
            return false;
        }
    };
}

macro_rules! match_comp {
    ($a:expr, $b:expr, $name:ident) => {
        match_ele!($a, $b, Component, $name)
    };
}

fn matches(a: &Uri, b: &Uri) -> bool {
    match_ele!(a, b, CpeType, part);
    match_comp!(a, b, vendor);
    match_comp!(a, b, product);
    match_comp!(a, b, version);
    match_comp!(a, b, update);
    match_comp!(a, b, edition);
    match_ele!(a, b, Language, *language);
    match_comp!(a, b, sw_edition);
    match_comp!(a, b, target_sw);
    match_comp!(a, b, target_hw);
    match_comp!(a, b, other);

    true
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();

    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    run(&multi).await
}
