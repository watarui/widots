use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use crate::error::app_error::AppError;
use crate::models::yaml::Yaml;

pub trait YamlOperations {
    fn validate_filename(&self, path: &Path) -> Result<(), AppError>;
    fn parse(&self, path: &Path) -> Result<Yaml, AppError>;
}

pub struct YamlParser;

impl Default for YamlParser {
    fn default() -> Self {
        YamlParser
    }
}

impl YamlParser {
    pub fn new() -> Arc<Self> {
        Arc::new(YamlParser)
    }
}

impl YamlOperations for YamlParser {
    fn validate_filename(&self, path: &Path) -> Result<(), AppError> {
        match path.extension() {
            Some(ext) if ext == "yaml" || ext == "yml" => Ok(()),
            _ => Err(AppError::InvalidInput(
                "Invalid YAML file extension".to_string(),
            )),
        }
    }

    fn parse(&self, path: &Path) -> Result<Yaml, AppError> {
        if !path.exists() {
            return Err(AppError::FileNotFound(path.to_path_buf()));
        }
        let mut file = File::open(path).map_err(|e| AppError::Io(Arc::new(e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| AppError::Io(Arc::new(e)))?;

        serde_yaml::from_str(&contents).map_err(|e| AppError::Yaml(e.to_string()))
    }
}

pub fn validate_filename(
    yaml_parser: Arc<dyn YamlOperations>,
    path: &Path,
) -> Result<(), AppError> {
    yaml_parser.validate_filename(path)
}

pub fn parse(yaml_parser: Arc<dyn YamlOperations>, path: &Path) -> Result<Yaml, AppError> {
    yaml_parser.parse(path)
}
