use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Shell execution error: {0}")]
    ShellExecution(String),

    #[error("Unsupported OS: {0}")]
    UnsupportedOS(String),

    #[error("Logger error: {0}")]
    Logger(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid file name: {0}")]
    InvalidFilename(String),

    #[error("Directory not found")]
    DirectoryNotFound,

    #[error("Deployment error: {0}")]
    Deployment(String),

    #[error("Symlink error: {0}")]
    Symlink(String),
}
