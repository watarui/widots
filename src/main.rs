mod application;
mod constants;
mod domain;
mod error;
mod infrastructure;
mod models;
mod presentation;
mod utils;

use application::service_provider::ProductionServiceProvider;
use clap::Parser;
use error::AppError;
use log::LevelFilter;

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

    let services = ProductionServiceProvider::new().await?;

    presentation::cli::run(args, &services).await
}
