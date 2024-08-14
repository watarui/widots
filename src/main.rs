mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod models;
mod presentation;
mod utils;

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

    logger::setup_logger(log_level)?;

    let config = AppConfig::new().await?;

    presentation::cli::run(args, &config).await
}
