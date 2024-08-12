// use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("YAML parse error: {0}")]
    YamlParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Shell execution error: {0}")]
    ShellExecutionError(String),

    #[error("Unsupported OS: {0}")]
    UnsupportedOS(String),

    #[error("Path error: {0}")]
    PathError(String),

    // #[error("Link error: {0}")]
    // LinkError(String),
    #[error("Logger error: {0}")]
    LoggerError(String),
    // #[error("File not found: {0}")]
    // FileNotFound(PathBuf),

    // #[error("Homebrew error: {0}")]
    // HomebrewError(String),

    // #[error("Fish shell error: {0}")]
    // FishError(String),

    // #[error("VSCode error: {0}")]
    // VSCodeError(String),

    // #[error("Unexpected error: {0}")]
    // UnexpectedError(String),
}

impl AppError {
    pub fn with_context<C: std::fmt::Display>(self, context: C) -> Self {
        match self {
            AppError::IoError(e) => AppError::IoError(format!("{}: {}", context, e)),
            AppError::YamlParseError(e) => AppError::YamlParseError(format!("{}: {}", context, e)),
            AppError::ConfigError(e) => AppError::ConfigError(format!("{}: {}", context, e)),
            AppError::ShellExecutionError(e) => {
                AppError::ShellExecutionError(format!("{}: {}", context, e))
            }
            AppError::UnsupportedOS(e) => AppError::UnsupportedOS(format!("{}: {}", context, e)),
            AppError::PathError(e) => AppError::PathError(format!("{}: {}", context, e)),
            // AppError::LinkError(e) => AppError::LinkError(format!("{}: {}", context, e)),
            AppError::LoggerError(e) => AppError::LoggerError(format!("{}: {}", context, e)),
            // AppError::FileNotFound(p) => AppError::FileNotFound(p),
            // AppError::HomebrewError(e) => AppError::HomebrewError(format!("{}: {}", context, e)),
            // AppError::FishError(e) => AppError::FishError(format!("{}: {}", context, e)),
            // AppError::VSCodeError(e) => AppError::VSCodeError(format!("{}: {}", context, e)),
            // AppError::UnexpectedError(e) => {
            //     AppError::UnexpectedError(format!("{}: {}", context, e))
            // }
        }
    }
}
