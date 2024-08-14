pub mod application;
pub mod constants;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;
pub mod presentation;
pub mod utils;

use clap::Parser;
use error::AppError;
use log::LevelFilter;

use crate::application::AppConfig;
use crate::presentation::cli::Args;
use crate::utils::logger;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    logger::setup_logger(log_level).map_err(|e| AppError::Logger(e.to_string()))?;

    let config = AppConfig::new().await?;

    presentation::cli::run(args, &config).await
}
