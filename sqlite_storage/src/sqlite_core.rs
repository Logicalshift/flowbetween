use flo_animation::storage::*;

use rusqlite;

const BASE_DATA_DEFN: &[u8]          = include_bytes!["../sql/flo_storage.sql"];

///
/// The SQLite core stores the synchronous data for the SQLite database
///
pub (super) struct SqliteCore {
    /// The database connection
    connection: rusqlite::Connection,

    /// If the core has encountered an error it can't recover from, this is what it is
    error: Option<(StorageError, String)>
}

impl SqliteCore {
    ///
    /// Creates a new core from a SQLite connection
    ///
    pub fn new(connection: rusqlite::Connection) -> SqliteCore {
        SqliteCore {
            connection: connection,
            error:      None
        }
    }

    ///
    /// Checks a SQLite result for an error and sets the error flag if one has occurred
    ///
    fn check_error<T>(&mut self, val: Result<T, rusqlite::Error>) -> Result<T, rusqlite::Error> {
        match val {
            Err(e) => {
                self.error = Some((StorageError::General, e.to_string()));
                Err(e)
            },

            Ok(r)   => Ok(r)
        }
    }

    ///
    /// When the connection is blank, initialises the data
    ///
    pub fn initialize(&mut self) -> Result<(), rusqlite::Error> {
        let defn = String::from_utf8_lossy(BASE_DATA_DEFN);

        self.check_error(self.connection.execute_batch(&defn))
    }

    ///
    /// Runs some commands on this storage database
    ///
    pub fn run_commands(&mut self, commands: Vec<StorageCommand>) -> Vec<StorageResponse> {
        // If we're in an error state, then the result is just to indicate that we can't continue
        if let Some((_err, msg)) = self.error.as_ref() {
            return vec![StorageResponse::Error(StorageError::CannotContinueAfterError, msg.clone())];
        }

        // Process each of the commands in turn and flatten to a single response
        let result = commands.into_iter()
            .map(|cmd| self.run_command(cmd))
            .collect::<Result<Vec<Vec<StorageResponse>>, _>>()
            .map(|vec_of_vec| vec_of_vec.into_iter().flatten().collect());

        match self.check_error(result) {
            Err(err)    => vec![StorageResponse::Error(StorageError::General, err.to_string())],
            Ok(result)  => result
        }
    }

    ///
    /// Runs an individual command and returns the values to generate in the response
    ///
    pub fn run_command(&mut self, command: StorageCommand) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use self::StorageCommand::*;

        match command {
            WriteAnimationProperties(properties)                => { },
            ReadAnimationProperties                             => { },
            WriteEdit(edit)                                     => { },
            ReadHighestUnusedElementId                          => { },
            ReadEditLogLength                                   => { },
            ReadEdits(edit_range)                               => { },
            WriteElement(element_id, value)                     => { },
            ReadElement(element_id)                             => { },
            DeleteElement(element_id)                           => { },
            AddLayer(layer_id, properties)                      => { },
            DeleteLayer(layer_id)                               => { },
            ReadLayers                                          => { },
            WriteLayerProperties(layer_id, properties)          => { },
            ReadLayerProperties(layer_id)                       => { },
            AddKeyFrame(layer_id, when)                         => { },
            DeleteKeyFrame(layer_id, when)                      => { },
            ReadKeyFrames(layer_id, time_range)                 => { },
            AttachElementToLayer(layer_id, element_id, when)    => { },
            DetachElementFromLayer(element_id)                  => { },
            ReadElementAttachments(element_id)                  => { },
            ReadElementsForKeyFrame(layer_id, when)             => { },
            WriteLayerCache(layer_id, when, cache_type, value)  => { },
            DeleteLayerCache(layer_id, when, cache_type)        => { },
            ReadLayerCache(layer_id, when, cache_type)          => { },
        }

        Ok(vec![])
    }
}
