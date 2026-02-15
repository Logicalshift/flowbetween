use super::super::error::*;

use rusqlite;

impl From<rusqlite::Error> for CanvasError {
    fn from(value: rusqlite::Error) -> Self {
        CanvasError::UnexpectedStorageError(value.to_string())
    }
}
