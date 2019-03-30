use super::error::*;
use std::result;

pub type Result<T> = result::Result<T, SqliteAnimationError>;
