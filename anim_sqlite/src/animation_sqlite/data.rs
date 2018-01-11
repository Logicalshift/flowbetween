use super::*;
use super::super::db::*;

use rusqlite::*;
use std::path::Path;

impl SqliteAnimation {
    ///
    /// Creates a new in-memory animation
    /// 
    pub fn new_in_memory() -> SqliteAnimation {
        let db = AnimationDb::new();
        db.setup();

        SqliteAnimation {
            db: db
        }
    }

    ///
    /// Creates an animation in a file
    /// 
    pub fn new_with_file<P: AsRef<Path>>(path: P) -> Result<SqliteAnimation> {
        let db = AnimationDb::new_from_connection(Connection::open_with_flags(path, SQLITE_OPEN_CREATE)?);
        db.setup();

        Ok(SqliteAnimation {
            db: db
        })
    }
}
