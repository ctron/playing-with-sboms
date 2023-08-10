use super::Task;
use indicatif::ProgressBar;
use spdx_rs::models::SPDX;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct UniqueNames {
    set: BTreeSet<String>,
}

impl Task for UniqueNames {
    fn process(&mut self, _: &ProgressBar, spdx: &SPDX) -> anyhow::Result<()> {
        self.set
            .insert(spdx.document_creation_information.document_name.clone());

        Ok(())
    }
}

impl Drop for UniqueNames {
    fn drop(&mut self) {
        println!("{} unique names", self.set.len());
        for i in &self.set {
            println!("{i}");
        }
    }
}
