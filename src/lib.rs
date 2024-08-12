pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;
pub mod logger;
pub mod models;
pub mod utils;

use core::{os::OSDetector, path::PathExpander, shell::ShellExecutor};
use std::sync::Arc;

use config::app_config::AppConfig;
use error::app_error::AppError;
use utils::yaml::YamlParser;

pub fn create_app_config() -> AppConfig {
    AppConfig::new(
        Arc::new(OSDetector::new()),
        Arc::new(PathExpander::new()),
        Arc::new(ShellExecutor::new()),
        YamlParser::new(),
    )
}

pub type Result<T> = std::result::Result<T, AppError>;
