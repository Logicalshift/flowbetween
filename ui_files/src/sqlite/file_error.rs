use rusqlite;

///
/// Errors that can happen when handling file lists
///
#[derive(Debug)]
pub enum FileListError {
    /// Files have a bad version number
    BadVersionNumber(String),

    /// SQLite error of some kind
    SqlError(rusqlite::Error)
}