use crate::models::yaml::{Link, Provision};
use crate::{error::AppError, models::yaml::Yaml};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use yaml_rust2::{Yaml as Yaml2, YamlLoader};

#[async_trait]
pub trait YamlOperations: Send + Sync {
    async fn parse(&self, path: &Path) -> Result<Yaml, AppError>;
}

pub struct YamlParser;

impl YamlParser {
    pub fn new() -> Self {
        Self
    }

    fn parse_link(&self, yaml: &Yaml2) -> Option<Link> {
        yaml["location"].as_str().map(|location| Link {
            location: PathBuf::from(location),
        })
    }

    fn parse_provision(&self, yaml: &Yaml2) -> Option<Provision> {
        let mode = yaml["mode"].as_str()?;
        let script = yaml["script"].as_str()?;
        Some(Provision {
            mode: mode.to_string(),
            script: script.to_string(),
        })
    }

    async fn parse_links(&self, yaml: &Yaml2) -> Option<Vec<Link>> {
        yaml.as_vec()?
            .iter()
            .filter_map(|yaml| self.parse_link(yaml))
            .collect::<Vec<_>>()
            .into()
    }

    async fn parse_provisions(&self, yaml: &Yaml2) -> Option<Vec<Provision>> {
        yaml.as_vec()?
            .iter()
            .filter_map(|yaml| self.parse_provision(yaml))
            .collect::<Vec<_>>()
            .into()
    }

    async fn parse_yaml(&self, yaml: &Yaml2) -> Yaml {
        Yaml {
            link: self.parse_links(&yaml["link"]).await,
            provision: self.parse_provisions(&yaml["provision"]).await,
        }
    }
}

#[async_trait]
impl YamlOperations for YamlParser {
    async fn parse(&self, path: &Path) -> Result<Yaml, AppError> {
        let mut file = File::open(path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let docs = YamlLoader::load_from_str(&contents)
            .map_err(|e| AppError::YamlParseError(e.to_string()))?;
        let doc = &docs[0];

        let yaml = self.parse_yaml(doc).await;

        Ok(yaml)
    }
}
