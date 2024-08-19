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

use crate::presentation::cli::{run, Args};
use crate::utils::logger;

pub async fn run_app(args: Args) -> Result<(), AppError> {
    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    logger::setup_logger(log_level).map_err(|e| AppError::Logger(e.to_string()))?;

    let services = ProductionServiceProvider::new().await?;

    run(args, &services).await
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();
    run_app(args).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_provider::TestServiceProvider;
    use std::io::ErrorKind;

    const APP_NAME: &str = "widots";

    #[tokio::test]
    async fn test_run_app() {
        let test_service_provider = TestServiceProvider::new(true);

        let args = Args::parse_from([APP_NAME, "-v", "link", "/nonexistent"]);

        let result = async {
            let log_level = match args.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            };

            logger::setup_logger(log_level).map_err(|e| AppError::Logger(e.to_string()))?;

            run(args, &test_service_provider).await
        }
        .await;

        assert!(
            result.is_err(),
            "Expected run_app to fail, but it succeeded"
        );

        if let Err(AppError::Io(io_error)) = result {
            assert_eq!(
                io_error.kind(),
                ErrorKind::NotFound,
                "Expected NotFound error"
            );
            assert!(
                io_error.to_string().contains("No such file or directory"),
                "Error message should mention 'No such file or directory'"
            );
        } else {
            panic!("Expected Io error, got: {:?}", result);
        }
    }

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from([APP_NAME, "-v", "link", "/src"]);
        assert_eq!(args.verbose, 1);

        let args = Args::parse_from([APP_NAME, "-vv", "link", "/src"]);
        assert_eq!(args.verbose, 2);

        let args = Args::parse_from([APP_NAME, "link", "/src"]);
        assert_eq!(args.verbose, 0);
    }
}
