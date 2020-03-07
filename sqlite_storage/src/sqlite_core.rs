use rusqlite;

const BASE_DATA_DEFN: &[u8]          = include_bytes!["../sql/flo_storage.sql"];

///
/// The SQLite core stores the synchronous data for the SQLite database
///
pub (super) struct SqliteCore {
    /// The database connection
    connection: rusqlite::Connection
}

impl SqliteCore {
    ///
    /// Creates a new core from a SQLite connection
    ///
    pub fn new(connection: rusqlite::Connection) -> SqliteCore {
        SqliteCore {
            connection: connection
        }
    }

    ///
    /// When the connection is blank, initialises the data
    ///
    pub fn initialize(&mut self) -> Result<(), rusqlite::Error> {
        let defn = String::from_utf8_lossy(BASE_DATA_DEFN);

        self.connection.execute_batch(&defn)
    }
}