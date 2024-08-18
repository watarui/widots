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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    const APP: &str = "widots";
    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([APP, "-v", "link", "/src"]);
        assert_eq!(args.verbose, 1);

        let args = Args::parse_from([APP, "-vv", "link", "/src"]);
        assert_eq!(args.verbose, 2);

        let args = Args::parse_from([APP, "link", "/src"]);
        assert_eq!(args.verbose, 0);
    }
}
