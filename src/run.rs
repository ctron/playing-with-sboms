use crate::tasks::Task;
use crate::walker::walk_sboms;
use indicatif::MultiProgress;

pub fn run_task<T: Task>(multi: &MultiProgress, task: &mut T) -> anyhow::Result<()> {
    let mut total = 0;

    walk_sboms(&multi, |pg, spdx| {
        total += 1;
        task.process(pg, spdx)
    })?;

    println!("Processed {total} SBOMs");

    Ok(())
}
