use crate::{error::AppError, models::config::Config};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait TomlOperations: Send + Sync {
    async fn parse(&self, path: &Path) -> Result<Config, AppError>;
}

pub struct TomlParser;

impl TomlParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TomlOperations for TomlParser {
    async fn parse(&self, path: &Path) -> Result<Config, AppError> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        toml::from_str(&contents).map_err(AppError::TomlParse)
    }
}
