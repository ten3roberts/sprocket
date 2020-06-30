use colorful::{Color, Colorful};
use log::{Level, LevelFilter, Metadata, Record};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();

        let color = match level {
            Level::Trace => Color::LightGray,
            Level::Debug => Color::LightGray,
            Level::Info => Color::Blue,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
        };

        println!(
            "[{}] {}: {}",
            level.to_string().color(color),
            record.target().split("::").last().unwrap_or("unknown"),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub type VerbosityLevel = log::LevelFilter;

pub fn init(level: LevelFilter) {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
        .unwrap_or_else(|e| eprintln!("Failed to initialize logger {}", e))
}
