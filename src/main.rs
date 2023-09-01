use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use playing_with_sboms::{run::run_task, tasks};

fn run(multi: &MultiProgress) -> anyhow::Result<()> {
    // run_task(multi, &mut tasks::unique_names::UniqueNames::default())?;
    // run_task(multi, &mut tasks::unique_main::UniqueMainPackages::default())?;
    run_task(multi, &mut tasks::main_cpe::MainCpe::default())?;
    /*
    run_task(
        multi,
        &mut tasks::main_cpe_db::MainCpeDb::new(playing_with_sboms::utils::cpe_dictionary::load(multi)?),
    )?;*/
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();

    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    run(&multi)
}
