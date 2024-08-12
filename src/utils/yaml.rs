use crate::error::AppError;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait YamlOperations: Send + Sync {
    // todo implement
    async fn _parse<T: DeserializeOwned>(&self, path: &Path) -> Result<T, AppError>;
}

pub struct YamlParser;

impl YamlParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl YamlOperations for YamlParser {
    async fn _parse<T: DeserializeOwned>(&self, path: &Path) -> Result<T, AppError> {
        let mut file = File::open(path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        serde_yaml::from_str(&contents).map_err(|e| AppError::YamlParseError(e.to_string()))
    }
}
