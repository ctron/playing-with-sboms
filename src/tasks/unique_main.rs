use super::Task;
use indicatif::ProgressBar;
use spdx_rs::models::SPDX;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct UniqueMainPackages {
    map: BTreeMap<String, usize>,
}

impl Task for UniqueMainPackages {
    fn process(&mut self, _progress: &ProgressBar, spdx: &SPDX) -> anyhow::Result<()> {
        let mut packages = spdx
            .document_creation_information
            .document_describes
            .iter()
            .map(|s| s.as_str())
            .collect::<BTreeSet<_>>();

        if packages.is_empty() {
            packages.insert("MISSING-MAIN-PACKAGE");
        }

        for package in packages {
            let pi = spdx
                .package_information
                .iter()
                .find(|p| &p.package_spdx_identifier == package);
            match pi {
                Some(package) => {
                    //progress.println(format!("Package: {}", package.package_name))
                    let key = &package.package_name;
                    *self.map.entry(key.clone()).or_default() += 1;
                }
                None => {
                    log::warn!("Missing package: {package}");
                    *self.map.entry(package.to_string()).or_default() += 1;
                }
            }
        }

        Ok(())
    }
}

impl Drop for UniqueMainPackages {
    fn drop(&mut self) {
        println!("{} unique entries", self.map.len());
        for (k, v) in &self.map {
            println!("{k}: {v}");
        }
    }
}
