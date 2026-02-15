use super::super::error::*;

use rusqlite;
use postcard;

impl From<rusqlite::Error> for CanvasError {
    fn from(value: rusqlite::Error) -> Self {
        CanvasError::UnexpectedStorageError(value.to_string())
    }
}

impl From<postcard::Error> for CanvasError {
    fn from(value: postcard::Error) -> Self {
        CanvasError::SerializationError(value.to_string())
    }
}
