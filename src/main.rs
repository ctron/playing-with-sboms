mod run;
mod tasks;
mod utils;
mod walker;

use crate::run::run_task;
use crate::tasks::Task;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;

fn run(multi: &MultiProgress) -> anyhow::Result<()> {
    // run_task(multi, tasks::unique_names::UniqueNames::default())?;
    // run_task(multi, tasks::unique_main::UniqueMainPackages::default())?;
    run_task(
        multi,
        tasks::main_cpe_db::MainCpeDb::new(utils::cpe_dictionary::load(multi)?),
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();

    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    run(&multi)
}
