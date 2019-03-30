use rusqlite;

///
/// Errors that can result from an operation on a SQLite animation
///
#[derive(Debug)]
pub enum SqliteAnimError {
    /// The version of this data is not supported by this version of FlowBetween
    UnsupportedVersionNumber(i64),

    /// Cannot upgrade from this version of the file format (it is too old or from a developmental build)
    CannotUpgradeVersionTooOld(i64),

    /// Cannot open this file format because it contains a patch that is not supported by this version of FlowBetween
    UnsupportedFormatPatch(String),

    /// SQLite error of some kind
    SqlError(rusqlite::Error)
}

impl From<rusqlite::Error> for SqliteAnimError {
    fn from(err: rusqlite::Error) -> SqliteAnimError {
        SqliteAnimError::SqlError(err)
    }
}
