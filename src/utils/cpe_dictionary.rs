use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::BufReader;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Generator {
    pub product_name: String,
    pub product_version: String,
    pub schema_version: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CpeItem {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default)]
    pub title: Vec<Title>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Title {
    #[serde(rename = "@lang")]
    pub lang: String,
    #[serde(rename = "$value")]
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CpeList {
    pub generator: Generator,
    #[serde(rename = "cpe-item", default)]
    pub items: Vec<CpeItem>,
}

pub fn load(multi: &MultiProgress) -> anyhow::Result<CpeList> {
    log::info!("Loading CPE dictionary");

    let file = std::fs::File::open("data/official-cpe-dictionary_v2.3.xml.gz")?;

    let style = ProgressStyle::with_template("{msg:20} {wide_bar} {bytes}/{total_bytes}")?;
    let progress = multi
        .add(ProgressBar::new(file.metadata()?.len()).with_style(style.clone()))
        .with_message("Loading CPE database");

    let decoder = flate2::read::GzDecoder::new(BufReader::new(progress.wrap_read(file)));
    let list: CpeList = quick_xml::de::from_reader(BufReader::new(decoder))?;

    log::info!("Loaded {} definitions", list.items.len());

    Ok(list)
}

impl CpeList {
    pub fn lookup(&self, cpe: &str) -> Option<&CpeItem> {
        self.items.iter().find(|item| item.name == cpe)
    }

    pub fn lookup_title(&self, cpe: &str, lang: &str) -> Option<&str> {
        self.lookup(cpe).and_then(|cpe| {
            cpe.title
                .iter()
                .find(|title| title.lang == lang)
                .map(|title| title.title.as_str())
        })
    }
}
