use flo_animation::*;

use rusqlite;
use rusqlite::Error;

///
/// Errors that can result from an operation on a SQLite animation
///
#[derive(Debug)]
pub enum SqliteAnimationError {
    /// No results were retrieved for a query (used instead of the SqlError version of this for convenience)
    QueryReturnedNoRows,

    /// The version of this data is not supported by this version of FlowBetween
    UnsupportedVersionNumber(i64),

    /// Cannot upgrade from this version of the file format (it is too old or from a developmental build)
    CannotUpgradeVersionTooOld(i64),

    /// Cannot open this file format because it contains a patch that is not supported by this version of FlowBetween
    UnsupportedFormatPatch(String),

    /// SQLite error of some kind
    SqlError(rusqlite::Error),

    /// An expected element was missing
    MissingElementId(ElementId),

    /// An element with the specified ID was not of the correct type
    UnexpectedElementType(ElementId),
}

impl From<rusqlite::Error> for SqliteAnimationError {
    fn from(err: rusqlite::Error) -> SqliteAnimationError {
        match err {
            Error::QueryReturnedNoRows  => SqliteAnimationError::QueryReturnedNoRows,
            err                         => SqliteAnimationError::SqlError(err)
        }
    }
}
