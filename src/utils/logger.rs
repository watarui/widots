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
    use log::LevelFilter;

    #[test]
    fn test_setup_logger() {
        let result = setup_logger(LevelFilter::Info);
        assert!(result.is_ok(), "Logger setup failed");

        let result = setup_logger(LevelFilter::Debug);
        assert!(result.is_ok(), "Second logger setup failed");
    }
}
