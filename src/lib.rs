//! Widots: A dotfile manager
//!
//! This library provides the core functionality for managing dotfiles,
//! including linking, materializing, and running configuration scripts.

pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;
pub mod logger;
pub mod models;
pub mod utils;

use config::app_config::AppConfig;
use error::app_error::AppError;

/// Creates a new AppConfig instance
pub async fn create_app_config() -> std::result::Result<AppConfig, AppError> {
    AppConfig::new().await
}

/// Alias for Result<T, AppError>
pub type Result<T> = std::result::Result<T, AppError>;
