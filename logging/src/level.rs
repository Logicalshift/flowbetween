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
