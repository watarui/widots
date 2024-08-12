use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::app_error::AppError;
use crate::models::yaml::Yaml;

/// Provides operations for YAML file handling.
#[async_trait]
pub trait YamlOperations: Send + Sync {
    /// Validates the filename of a YAML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML file
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the validation.
    async fn validate_filename(&self, path: &Path) -> Result<(), AppError>;

    /// Parses a YAML file into a serde_yaml::Value.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML file
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed YAML data,
    /// or an `AppError` if parsing fails.
    async fn parse(&self, path: &Path) -> Result<Yaml, AppError>;
}

/// Implements YAML file operations.
pub struct YamlParser;

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl YamlParser {
    /// Creates a new `YamlParser` instance.
    pub fn new() -> Self {
        YamlParser
    }
}

#[async_trait]
impl YamlOperations for YamlParser {
    async fn validate_filename(&self, path: &Path) -> Result<(), AppError> {
        match path.extension() {
            Some(ext) if ext == "yaml" || ext == "yml" => Ok(()),
            _ => Err(AppError::InvalidInput(
                "Invalid YAML file extension".to_string(),
            )),
        }
    }

    async fn parse(&self, path: &Path) -> Result<Yaml, AppError> {
        if !path.exists() {
            return Err(AppError::FileNotFound(path.to_path_buf()));
        }
        let mut file = File::open(path).await.map_err(AppError::Io)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AppError::Io)?;

        serde_yaml::from_str(&contents).map_err(|e| AppError::Yaml(e.to_string()))
    }
}
