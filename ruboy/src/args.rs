use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(author, about, version)]
pub(crate) struct CLIArgs {
    pub rom: PathBuf,

    /// The verbosity of the logger
    #[cfg(not(debug_assertions))]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Warn)]
    pub verbosity: LogLevel,

    /// The verbosity of the logger
    #[cfg(debug_assertions)]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Info)]
    pub verbosity: LogLevel,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum LogLevel {
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
