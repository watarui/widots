use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
    INIT.call_once(|| {
        internal_setup_logger(level).expect("Failed to initialize logger");
    });
    Ok(())
}

fn internal_setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use log::{Level, Record};

    #[test]
    fn test_setup_logger() {
        let result = setup_logger(LevelFilter::Info);
        assert!(result.is_ok(), "Logger setup failed");

        let result = setup_logger(LevelFilter::Debug);
        assert!(result.is_ok(), "Second logger setup failed");
    }

    #[test]
    fn test_log_format() {
        let colors = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::Green)
            .debug(Color::Blue)
            .trace(Color::Magenta);

        let record = Record::builder()
            .args(format_args!("Test message"))
            .level(Level::Info)
            .target("test_target")
            .build();

        let formatted_log = format!(
            "{}[{}][{}] {}",
            Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            record.target(),
            colors.color(record.level()),
            record.args()
        );

        assert!(formatted_log.contains(&Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string()));
        assert!(formatted_log.contains("[test_target]"));
        assert!(formatted_log.contains("INFO"));
        assert!(formatted_log.contains("Test message"));
    }
}
