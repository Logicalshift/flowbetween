use rusqlite;

///
/// Errors that can happen when handling file lists
///
#[derive(Debug)]
pub enum FileListError {
    /// Files have a bad version number
    BadVersionNumber(String),

    /// No upgrade script exists for the file list version
    CannotUpgradeVersion,

    /// SQLite error of some kind
    SqlError(rusqlite::Error)
}

impl From<rusqlite::Error> for FileListError {
    fn from(err: rusqlite::Error) -> FileListError {
        FileListError::SqlError(err)
    }
}
