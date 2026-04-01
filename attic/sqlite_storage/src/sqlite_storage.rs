use super::sqlite_core::*;

use flo_animation::storage::*;

use ::desync::*;
use rusqlite;
use futures::*;

use std::sync::*;
use std::path::{Path};

///
/// Stores an animation using a SQLite database
///
pub struct SqliteAnimationStorage {
    /// The core where the data resides
    core: Arc<Desync<SqliteCore>>
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
        let core    = Arc::new(Desync::new(core));

        // Initialise it (in the background)
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
        let core    = Arc::new(Desync::new(core));

        // Create the storage object
        SqliteAnimationStorage {
            core:   core
        }
    }

    ///
    /// Opens an existing database file
    ///
    pub fn open_file(path: &Path) -> Result<SqliteAnimationStorage, rusqlite::Error> {
        let connection  = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(Self::from_connection(connection))
    }

    ///
    /// Creates a new animation at the specified path
    ///
    pub fn new_with_file(path: &Path) -> Result<SqliteAnimationStorage, rusqlite::Error> {
        let connection  = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE)?;
        Ok(Self::new_from_connection(connection))
    }

    ///
    /// Creaters a SQLite storage object in memory
    ///
    pub fn new_in_memory() -> Result<SqliteAnimationStorage, rusqlite::Error> {
        Ok(Self::new_from_connection(rusqlite::Connection::open_in_memory()?))
    }

    ///
    /// Returns the responses for a stream of commands
    ///
    pub fn get_responses<CommandStream: 'static+Send+Unpin+Stream<Item=Vec<StorageCommand>>>(&self, commands: CommandStream) -> impl Send+Unpin+Stream<Item=Vec<StorageResponse>> {
        pipe(Arc::clone(&self.core), commands, |core, commands| {
            future::ready(core.run_commands(commands)).boxed()
        })
    }
}
