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

pub fn load_cpe_dictionary() -> anyhow::Result<CpeList> {
    let file = std::fs::File::open("data/official-cpe-dictionary_v2.3.xml.gz")?;
    let decoder = flate2::read::GzDecoder::new(BufReader::new(file));
    Ok(quick_xml::de::from_reader(BufReader::new(decoder))?)
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
