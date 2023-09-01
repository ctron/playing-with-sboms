use super::Task;
use indicatif::ProgressBar;
use spdx_rs::models::SPDX;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct MainCpe {
    pub map: BTreeMap<String, usize>,
}

impl Task for MainCpe {
    fn process(&mut self, _progress: &ProgressBar, spdx: &SPDX) -> anyhow::Result<()> {
        let packages = spdx
            .document_creation_information
            .document_describes
            .iter()
            .map(|s| s.as_str())
            .collect::<BTreeSet<_>>();

        if packages.is_empty() {
            *self
                .map
                .entry("MISSING-MAIN-PACKAGE".to_string())
                .or_default() += 1;
        } else {
            for package in packages {
                let pi = spdx
                    .package_information
                    .iter()
                    .find(|p| &p.package_spdx_identifier == package);
                match pi {
                    Some(package) => {
                        let mut num = 0;
                        for ext in &package.external_reference {
                            if ext.reference_type == "cpe22Type" {
                                *self.map.entry(ext.reference_locator.clone()).or_default() += 1;
                                num += 1;
                            }
                        }
                        if num == 0 {
                            *self.map.entry("NO-CPE".to_string()).or_default() += 1;
                        }
                    }
                    None => {
                        log::warn!("Missing package: {package}");
                        *self
                            .map
                            .entry("INVALID-PACKAGE-ID".to_string())
                            .or_default() += 1;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for MainCpe {
    fn drop(&mut self) {
        println!("{} unique entries", self.map.len());
        for (k, v) in &self.map {
            println!("{k}: {v}");
        }
    }
}
