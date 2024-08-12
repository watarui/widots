use std::{io::stdout, path::Path};

use chrono::Local;
use fern::{Dispatch, InitError};
use log::LevelFilter;

pub struct Log;

pub trait Logger {
    fn setup_logger(&self, level: LevelFilter) -> Result<(), InitError>;
}

impl Default for Log {
    fn default() -> Self {
        Log::new()
    }
}

impl Log {
    fn new() -> Self {
        Log
    }
}

impl Logger for Log {
    fn setup_logger(&self, level: LevelFilter) -> Result<(), InitError> {
        Dispatch::new()
            .format(move |out, message, record| {
                // Transform the file path into a relative path from src or tests
                let file_path = record.file().unwrap_or("unknown");
                let relative_path = Path::new(file_path)
                    .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .unwrap_or_else(|_| Path::new(file_path));

                out.finish(format_args!(
                    "{} [{}] {}:{} - {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    relative_path.display(),
                    record.line().unwrap_or(0),
                    message
                ))
            })
            // Set the maximum log level
            .level(level)
            // Output to stdout
            .chain(stdout())
            .apply()?;
        Ok(())
    }
}

pub fn setup_logger(level: LevelFilter) -> Result<(), InitError> {
    Log.setup_logger(level)
}
