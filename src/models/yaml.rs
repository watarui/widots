use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Yaml {
    #[serde(default)]
    pub link: Option<Vec<Link>>,
    #[serde(default)]
    pub provision: Option<Vec<Provision>>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub location: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Provision {
    pub mode: String,
    pub script: String,
}
