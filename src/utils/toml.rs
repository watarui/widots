use crate::{error::AppError, models::config::Config};
use async_trait::async_trait;
use std::path::Path;
#[cfg(test)]
use tempfile::NamedTempFile;
#[cfg(test)]
use tokio::fs;
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

#[tokio::test]
async fn test_parse_toml() -> Result<(), AppError> {
    let toml_content = r#"
        [[link]]
        location = "/path/to/dotfiles"

        [[provision]]
        mode = "macos"
        script = "echo 'Hello, macOS!'"
    "#;

    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_owned();
    fs::write(&temp_path, toml_content).await?;

    let toml_parser = TomlParser::new();
    let config = toml_parser.parse(Path::new(&temp_path)).await?;

    assert!(config.link.is_some());
    assert!(config.provision.is_some());
    assert_eq!(config.link.unwrap().len(), 1);
    assert_eq!(config.provision.unwrap().len(), 1);

    Ok(())
}
