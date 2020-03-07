use flo_animation::storage::*;

use rusqlite;
use rusqlite::{NO_PARAMS};

use std::ops::{Range};
use std::time::{Duration};

const BASE_DATA_DEFN: &[u8]          = include_bytes!["../sql/flo_storage.sql"];

///
/// The SQLite core stores the synchronous data for the SQLite database
///
pub (super) struct SqliteCore {
    /// The database connection
    connection: rusqlite::Connection,

    /// If the core has encountered an error it can't recover from, this is what it is
    error: Option<(StorageError, String)>,
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

        let result = match command {
            WriteAnimationProperties(properties)                => { self.write_animation_properties(properties) },
            ReadAnimationProperties                             => { self.read_animation_properties() },
            WriteEdit(edit)                                     => { self.write_edit(edit) },
            ReadHighestUnusedElementId                          => { self.read_highest_unused_element_id() },
            ReadEditLogLength                                   => { self.read_edit_log_length() },
            ReadEdits(edit_range)                               => { self.read_edits(edit_range) },
            WriteElement(element_id, value)                     => { self.write_element(element_id, value) },
            ReadElement(element_id)                             => { self.read_element(element_id) },
            DeleteElement(element_id)                           => { self.delete_element(element_id) },
            AddLayer(layer_id, properties)                      => { self.add_layer(layer_id, properties) },
            DeleteLayer(layer_id)                               => { self.delete_layer(layer_id) },
            ReadLayers                                          => { self.read_layers() },
            WriteLayerProperties(layer_id, properties)          => { self.add_layer(layer_id, properties) },
            ReadLayerProperties(layer_id)                       => { self.read_layer_properties(layer_id) },
            AddKeyFrame(layer_id, when)                         => { unimplemented!() },
            DeleteKeyFrame(layer_id, when)                      => { unimplemented!() },
            ReadKeyFrames(layer_id, time_range)                 => { unimplemented!() },
            AttachElementToLayer(layer_id, element_id, when)    => { unimplemented!() },
            DetachElementFromLayer(element_id)                  => { unimplemented!() },
            ReadElementAttachments(element_id)                  => { unimplemented!() },
            ReadElementsForKeyFrame(layer_id, when)             => { unimplemented!() },
            WriteLayerCache(layer_id, when, cache_type, value)  => { unimplemented!() },
            DeleteLayerCache(layer_id, when, cache_type)        => { unimplemented!() },
            ReadLayerCache(layer_id, when, cache_type)          => { unimplemented!() },
        };

        self.check_error(result)
    }

    ///
    /// Converts a Duration to a microseconds value which we can store in the database
    ///
    fn time_to_int(time: Duration) -> i64 {
        time.as_micros() as i64
    }

    ///
    /// Converts a microseconds value from the database to a Duration value
    ///
    fn int_to_time(time: i64) -> Duration {
        if time < 0 {
            Duration::from_micros(0)
        } else {
            Duration::from_micros(time as u64)
        }
    }

    ///
    /// Updates the animation properties for this animation
    ///
    fn write_animation_properties(&mut self, properties: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO AnimationProperties (PropertyId, Value) VALUES (0, ?);")?;
        write.execute(&[properties])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Reads the currently set animation properties (if any)
    ///
    fn read_animation_properties(&mut self) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read = self.connection.prepare_cached("SELECT Value FROM AnimationProperties WHERE PropertyId = 0;")?;

        match read.query_row(NO_PARAMS, |row| row.get(0)) {
            Ok(properties)              => Ok(vec![StorageResponse::AnimationProperties(properties)]),
            Err(QueryReturnedNoRows)    => Ok(vec![StorageResponse::NotFound]),
            Err(other)                  => Err(other)
        }
    }

    ///
    /// Updates the animation properties for this animation
    ///
    fn write_edit(&mut self, edit: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut write   = self.connection.prepare_cached("INSERT INTO EditLog (Edit) VALUES (?);")?;
        write.execute(&[edit])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Updates the animation properties for this animation
    ///
    fn read_edit_log_length(&mut self) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut read    = self.connection.prepare_cached("SELECT COALESCE(MAX(EditId), 0) FROM EditLog;")?;
        let count       = read.query_row(NO_PARAMS, |row| row.get::<_, i64>(0))?;

        Ok(vec![StorageResponse::NumberOfEdits(count as usize)])
    }

    ///
    /// Updates the animation properties for this animation
    ///
    fn read_highest_unused_element_id(&mut self) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read    = self.connection.prepare_cached("SELECT COALESCE(MAX(ElementId)+1, 0) FROM Elements;")?;
        let count       = read.query_row(NO_PARAMS, |row| row.get::<_, i64>(0));

        match count {
            Ok(count)                   => Ok(vec![StorageResponse::HighestUnusedElementId(count)]),
            Err(QueryReturnedNoRows)    => Ok(vec![StorageResponse::HighestUnusedElementId(0)]),
            Err(err)                    => Err(err)
        }
    }

    ///
    /// Updates the animation properties for this animation
    ///
    fn read_edits(&mut self, range: Range<usize>) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut read    = self.connection.prepare_cached("SELECT EditId, Edit FROM EditLog WHERE EditId >= ? AND EditId < ? ORDER BY EditId ASC;")?;
        let edits       = read.query_map(&[(range.start as i64)+1, (range.end as i64)+1], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
        let edits       = edits.map(|row| row.map(|(edit_id, edit)| StorageResponse::Edit((edit_id-1) as usize, edit)));

        Ok(edits.collect::<Result<_, _>>()?)
    }

    ///
    /// Writes data for an element
    ///
    fn write_element(&mut self, element_id: i64, element: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO Elements (ElementId, Element) VALUES (?, ?);")?;
        write.execute(params![element_id, element])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Reads data for an element
    ///
    fn read_element(&mut self, element_id: i64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read    = self.connection.prepare_cached("SELECT Element FROM Elements WHERE ElementId = ?;")?;
        let element     = read.query_row(&[element_id], |row| row.get(0));

        match element {
            Ok(element)                 => Ok(vec![StorageResponse::Element(element_id, element)]),
            Err(QueryReturnedNoRows)    => Ok(vec![StorageResponse::NotFound]),
            Err(err)                    => Err(err)
        }
    }

    ///
    /// Deletes an element from the database
    ///
    fn delete_element(&mut self, element_id: i64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let transaction = self.connection.transaction()?;

        {
            let mut delete  = transaction.prepare_cached("DELETE FROM ElementKeyframeAttachment WHERE ElementId = ?;")?;
            delete.execute(&[element_id])?;

            let mut delete  = transaction.prepare_cached("DELETE FROM Elements WHERE ElementId = ?;")?;
            delete.execute(&[element_id])?;
        }

        transaction.commit()?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Adds a new layer or updates its properties
    ///
    fn add_layer(&mut self, layer_id: u64, properties: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO Layers (LayerId, Layer) VALUES (?, ?);")?;
        write.execute(params![layer_id as i64, properties])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Deletes a layer from the database
    ///
    fn delete_layer(&mut self, layer_id: u64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let transaction = self.connection.transaction()?;

        {
            let mut delete  = transaction.prepare_cached("DELETE FROM ElementKeyframeAttachment WHERE LayerId = ?;")?;
            delete.execute(&[layer_id as i64])?;

            let mut delete  = transaction.prepare_cached("DELETE FROM LayerCache WHERE LayerId = ?;")?;
            delete.execute(&[layer_id as i64])?;

            let mut delete  = transaction.prepare_cached("DELETE FROM Layers WHERE LayerId = ?;")?;
            delete.execute(&[layer_id as i64])?;
        }

        transaction.commit()?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Reads data for an element
    ///
    fn read_layers(&mut self) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut read    = self.connection.prepare_cached("SELECT LayerId, Layer FROM Layers;")?;
        let layers      = read.query_map(NO_PARAMS, |row| Ok((row.get::<_, i64>(0)?, row.get(1)?)))?;
        let layers      = layers.map(|layer| layer.map(|(layer_id, layer)| StorageResponse::LayerProperties(layer_id as u64, layer)));

        Ok(layers.collect::<Result<_, _>>()?)
    }

    ///
    /// Reads data for an element
    ///
    fn read_layer_properties(&mut self, layer_id: u64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read            = self.connection.prepare_cached("SELECT LayerId, Layer FROM Layers WHERE LayerId = ?;")?;

        match read.query_row(&[layer_id as i64], |row| Ok((row.get::<_, i64>(0)?, row.get(1)?))) {
            Ok((layer_id, layer))       => Ok(vec![StorageResponse::LayerProperties(layer_id as u64, layer)]),
            Err(QueryReturnedNoRows)    => Ok(vec![StorageResponse::NotFound]),
            Err(other)                  => Err(other)
        }
    }
}
