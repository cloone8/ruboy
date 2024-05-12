use clap::ValueEnum;

pub mod dasm;

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    #[cfg(debug_assertions)]
    Debug,
    #[cfg(debug_assertions)]
    Trace,
}

impl From<LogLevel> for log::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            #[cfg(debug_assertions)]
            LogLevel::Debug => log::Level::Debug,
            #[cfg(debug_assertions)]
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            #[cfg(debug_assertions)]
            LogLevel::Debug => log::LevelFilter::Debug,
            #[cfg(debug_assertions)]
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}
