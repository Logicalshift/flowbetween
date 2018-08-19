use log;

///
/// Indicates the verbosity level of a log message
/// 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LogLevel {
    Debug,
    Verbose,
    Info,
    Warning,
    Error,
    Critical
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> LogLevel {
        match level {
            log::Level::Trace   => LogLevel::Debug,
            log::Level::Debug   => LogLevel::Verbose,
            log::Level::Info    => LogLevel::Info,
            log::Level::Warn    => LogLevel::Warning,
            log::Level::Error   => LogLevel::Error
        }
    }
}

impl Into<log::Level> for LogLevel {
    fn into(self) -> log::Level {
        match self {
            LogLevel::Debug     => log::Level::Trace,
            LogLevel::Verbose   => log::Level::Debug,
            LogLevel::Info      => log::Level::Info,
            LogLevel::Warning   => log::Level::Warn,
            LogLevel::Error     => log::Level::Error,
            LogLevel::Critical  => log::Level::Error
        }
    }
}