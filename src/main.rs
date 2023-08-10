mod tasks;
mod utils;
mod walker;

use crate::tasks::Task;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use walker::walk_sboms;

fn run(multi: &MultiProgress) -> anyhow::Result<()> {
    println!("Loading CPE dictionary ...");
    let cpe_dictionary = utils::cpe_dictionary::load_cpe_dictionary()?;
    // println!("{cpe_dictionary:#?}");
    println!("Loading CPE dictionary ... done!");

    // run_task(multi, tasks::unique_names::UniqueNames::default())?;
    // run_task(multi, tasks::unique_main::UniqueMainPackages::default())?;
    run_task(multi, tasks::main_cpe_db::MainCpeDb::new(cpe_dictionary))?;
    Ok(())
}

fn run_task<T: Task>(multi: &MultiProgress, mut task: T) -> anyhow::Result<()> {
    let mut total = 0;

    walk_sboms(&multi, |pg, spdx| {
        total += 1;
        task.process(pg, spdx)
    })?;

    println!("Processed {total} SBOMs");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();

    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    run(&multi)
}
