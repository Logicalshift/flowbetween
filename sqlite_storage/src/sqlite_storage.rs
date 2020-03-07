use super::sqlite_core::*;

use ::desync::*;
use rusqlite;

///
/// Stores an animation using a SQLite database
///
pub struct SqliteAnimationStorage {
    /// The core where the data resides
    core: Desync<SqliteCore>
}

impl SqliteAnimationStorage {
    ///
    /// Creates a new SQLite database from an existing connection
    /// 
    /// This will initialise the database, so use `from_connection` for the case where the core already exists
    ///
    pub fn new_from_connection(connection: rusqlite::Connection) -> SqliteAnimationStorage {
        // Create the core with the connection
        let core    = SqliteCore::new(connection);
        let core    = Desync::new(core);

        // Initialise it
        core.desync(|core| { core.initialize().ok(); });

        // Create the storage object
        SqliteAnimationStorage {
            core:   core
        }
    }

    ///
    /// Creates a SQLite storage from an existing database connection, which should already be initialised
    ///
    pub fn from_connection(connection: rusqlite::Connection) -> SqliteAnimationStorage {
        // Create the core with the connection
        let core    = SqliteCore::new(connection);
        let core    = Desync::new(core);

        // Create the storage object
        SqliteAnimationStorage {
            core:   core
        }
    }

    ///
    /// Creaters a SQLite storage object in memory
    ///
    pub fn new_in_memory() -> Result<SqliteAnimationStorage, rusqlite::Error> {
        Ok(Self::new_from_connection(rusqlite::Connection::open_in_memory()?))
    }
}
