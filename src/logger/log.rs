use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::io::stdout;

/// Sets up the logger for the application.
///
/// # Arguments
///
/// * `level` - The log level to use
///
/// # Returns
///
/// A `Result` indicating success or failure of logger setup.
pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}:{} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(level)
        .chain(stdout())
        .apply()?;
    Ok(())
}

/// Sets the global log level.
///
/// # Arguments
///
/// * `level` - The new log level to set
pub fn set_log_level(level: LevelFilter) {
    log::set_max_level(level);
}
