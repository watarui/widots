use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Home directory not found")]
    HomeDirectoryNotFound,

    #[error("Path error: {0}")]
    Path(String),

    #[error("Failed to strip path prefix: {0}")]
    PathStripPrefixError(#[from] std::path::StripPrefixError),

    #[error("Failed to canonicalize path: {0}")]
    PathCanonicalizationError(String),

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
    FileNotFound(std::path::PathBuf),

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

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Failed to add fish to /etc/shells")]
    AddFishToEtcShellsFailed,
}

impl AppError {
    pub fn with_context<C>(self, context: C) -> Self
    where
        C: std::fmt::Display,
    {
        match self {
            AppError::Io(err) => AppError::Io(std::io::Error::new(
                err.kind(),
                format!("{}: {}", context, err),
            )),
            AppError::Path(err) => AppError::Path(format!("{}: {}", context, err)),
            // Add similar handling for other error variants
            _ => self,
        }
    }
}
