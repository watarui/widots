use crate::error::AppError;
use crate::models::config::Config;
use crate::utils::toml::{TomlOperations, TomlParser};
use std::path::Path;
use tempfile::NamedTempFile;
use tokio::fs;

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
