use indicatif::ProgressBar;
use spdx_rs::models::SPDX;

pub mod main_cpe;
pub mod main_cpe_db;
pub mod unique_main;
pub mod unique_names;

pub trait Task {
    fn process(&mut self, progress: &ProgressBar, sbom: &SPDX) -> anyhow::Result<()>;
}
