use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(Arc<std::io::Error>),

    #[error("Home directory not found")]
    HomeDirectoryNotFound,

    #[error("Path error: {0}")]
    Path(String),

    #[error("Failed to strip path prefix: {0}")]
    PathStripPrefixError(#[from] std::path::StripPrefixError),

    #[error("Failed to canonicalize path: {0}")]
    PathCanonicalizationError(Arc<std::io::Error>),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("Failed to create temporary file: {0}")]
    TempFileCreationError(String),

    #[error("Failed to write script to temporary file: {0}")]
    ScriptWriteError(String),

    #[error("Bash execution failed: {0}")]
    BashExecutionFailed(String),

    #[error("Shell execution failed: {0}")]
    ShellExecutionFailed(String),

    #[error("Sudo execution failed: {0}")]
    SudoExecutionFailed(String),

    #[error("Which execution failed: {0}")]
    WhichExecutionError(String),

    #[error("Empty command")]
    EmptyCommand,

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Homebrew error: {0}")]
    Homebrew(String),

    #[error("Fish shell error: {0}")]
    Fish(String),

    #[error("Linking error: {0}")]
    Link(String),

    #[error("VSCode error: {0}")]
    VSCode(String),

    #[error("Deployment error: {0}")]
    Deployment(String),

    #[error("Logger error: {0}")]
    Logger(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("External command error: {0}")]
    ExternalCommand(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Command execution failed: {0}")]
    ExecuteCommandFailed(String),

    #[error("Failed to add fish to etc shells")]
    AddFishToEtcShellsFailed,
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(Arc::new(err))
    }
}
