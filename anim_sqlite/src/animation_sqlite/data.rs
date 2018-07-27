use super::*;

use rusqlite::*;
use std::path::Path;

impl SqliteAnimation {
    ///
    /// Creates a new in-memory animation
    /// 
    pub fn new_in_memory() -> SqliteAnimation {
        let db = AnimationDb::new();

        SqliteAnimation {
            db: db
        }
    }

    ///
    /// Creates an animation in a file
    /// 
    pub fn new_with_file<P: AsRef<Path>>(path: P) -> Result<SqliteAnimation> {
        let db = AnimationDb::new_from_connection(Connection::open_with_flags(path, SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE)?);

        Ok(SqliteAnimation {
            db: db
        })
    }

    ///
    /// Opens an existing file
    /// 
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<SqliteAnimation> {
        let connection  = Connection::open_with_flags(path, SQLITE_OPEN_READ_WRITE)?;
        let db          = AnimationDb::from_connection(connection);

        Ok(SqliteAnimation {
            db: db
        })
    }

    ///
    /// Takes an existing SQLite connection and creates a new animation in it
    /// 
    pub fn set_up_existing_database(sqlite: Connection) -> Result<SqliteAnimation> {
        let db = AnimationDb::new_from_connection(sqlite);
        Ok(SqliteAnimation {
            db: db
        })
    }

    ///
    /// Uses an existing SQLite connection with an animation in it to create an animation object
    /// 
    pub fn from_existing_database(sqlite: Connection) -> Result<SqliteAnimation> {
        let db = AnimationDb::from_connection(sqlite);
        Ok(SqliteAnimation {
            db: db
        })
    }
}
