use flo_animation::storage::*;

use rusqlite;
use rusqlite::{NO_PARAMS};

use std::i64;
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
            AddKeyFrame(layer_id, when)                         => { self.add_key_frame(layer_id, when) },
            DeleteKeyFrame(layer_id, when)                      => { self.delete_key_frame(layer_id, when) },
            ReadKeyFrames(layer_id, time_range)                 => { self.read_keyframes(layer_id, time_range) },
            AttachElementToLayer(layer_id, element_id, when)    => { self.attach_element_to_layer(layer_id, element_id, when) },
            DetachElementFromLayer(element_id)                  => { self.detach_element_from_layer(element_id) },
            ReadElementAttachments(element_id)                  => { self.read_element_attachments(element_id) },
            ReadElementsForKeyFrame(layer_id, when)             => { self.read_elements_for_key_frame(layer_id, when) },
            WriteLayerCache(layer_id, when, cache_type, value)  => { self.write_layer_cache(layer_id, when, cache_type, value) },
            DeleteLayerCache(layer_id, when, cache_type)        => { self.delete_layer_cache(layer_id, when, cache_type) },
            ReadLayerCache(layer_id, when, cache_type)          => { self.read_layer_cache(layer_id, when, cache_type) },
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

    ///
    /// Adds a new layer or updates its properties
    ///
    fn add_key_frame(&mut self, layer_id: u64, when: Duration) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO Keyframe (LayerId, TimeMicroseconds) VALUES (?, ?);")?;
        write.execute(params![layer_id as i64, Self::time_to_int(when)])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Adds a new layer or updates its properties
    ///
    fn delete_key_frame(&mut self, layer_id: u64, when: Duration) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let transaction = self.connection.transaction()?;

        {
            let time_microseconds   = Self::time_to_int(when);

            let mut delete  = transaction.prepare_cached("DELETE FROM ElementKeyframeAttachment WHERE LayerId = ? AND TimeMicroseconds = ?;")?;
            delete.execute(&[layer_id as i64, time_microseconds])?;

            let mut delete  = transaction.prepare_cached("DELETE FROM LayerCache WHERE LayerId = ? AND TimeMicroseconds = ?;")?;
            delete.execute(&[layer_id as i64, time_microseconds])?;

            let mut delete  = transaction.prepare_cached("DELETE FROM Keyframe WHERE LayerId = ? AND TimeMicroseconds = ?;")?;
            delete.execute(&[layer_id as i64, time_microseconds])?;
        }

        transaction.commit()?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Reads where the keyframe preceding or at the specified time is located
    ///
    fn read_previous_key_frame(&mut self, layer_id: u64, when_micros: i64) -> Result<Option<i64>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read_keyframe   = self.connection.prepare_cached("SELECT TimeMicroseconds FROM Keyframe WHERE LayerId = ? AND TimeMicroseconds <= ? ORDER BY TimeMicroseconds DESC LIMIT 1")?;
        let result              = read_keyframe.query_row(&[layer_id as i64, when_micros], |row| row.get::<_, i64>(0));

        match result {
            Ok(keyframe_time)           => Ok(Some(keyframe_time)),
            Err(QueryReturnedNoRows)    => Ok(None),
            Err(other)                  => Err(other)
        }
    }

    ///
    /// Reads where the keyframe at or following the specified time is located
    ///
    fn read_next_key_frame(&mut self, layer_id: u64, when_micros: i64) -> Result<Option<i64>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let mut read_keyframe   = self.connection.prepare_cached("SELECT TimeMicroseconds FROM Keyframe WHERE LayerId = ? AND TimeMicroseconds >= ? ORDER BY TimeMicroseconds ASC LIMIT 1")?;
        let result              = read_keyframe.query_row(&[layer_id as i64, when_micros], |row| row.get::<_, i64>(0));

        match result {
            Ok(keyframe_time)           => Ok(Some(keyframe_time)),
            Err(QueryReturnedNoRows)    => Ok(None),
            Err(other)                  => Err(other)
        }
    }

    ///
    /// Reads the keyframes that exist in a particular time range
    ///
    fn read_keyframes(&mut self, layer_id: u64, when: Range<Duration>) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        // Align the times to where the actual keyframes exist
        let start   = self.read_previous_key_frame(layer_id, Self::time_to_int(when.start))?;
        let end     = self.read_next_key_frame(layer_id, Self::time_to_int(when.end))?;

        // If the start was not found but the end was, then return not found
        let start = match (start, &end) {
            (None, Some(end))   => {
                if Some(*end) == self.read_next_key_frame(layer_id, 0)? {
                    // Picked a time range before the first keyframe
                    return Ok(vec![StorageResponse::NotInAFrame(Self::int_to_time(*end))]); 
                } else {
                    // Time range covers other frames
                    0
                }
            }
            (Some(start), _)    => { start },
            (None, _)           => { 0 }
        };

        // If no end was found, it's at the maximum possible time
        let mut end = end.unwrap_or(i64::MAX);

        // If the start and the end are the same then we need to read to the end of the current keyframe
        if start == end  {
            end = self.read_next_key_frame(layer_id, end+1)?.unwrap_or(i64::MAX);
        }

        // Read the keyframes that exist in this time
        let mut read_keyframes  = self.connection.prepare_cached("SELECT TimeMicroseconds FROM Keyframe WHERE LayerId = ? AND TimeMicroseconds >= ? AND TimeMicroseconds <= ? ORDER BY TimeMicroseconds ASC")?;
        let keyframes           = read_keyframes.query_map(&[layer_id as i64, start, end], |row| row.get::<_, i64>(0))?;

        let mut result          = vec![];
        let mut last_time       = None;
        for frame_time in keyframes {
            let frame_time = frame_time?;

            if let Some(start_time) = last_time.take() {
                // This is the end time of a keyframe
                let start_time  = Self::int_to_time(start_time);
                let end_time    = Self::int_to_time(frame_time);

                result.push(StorageResponse::KeyFrame(start_time, end_time));

                // Will also be the start of the next frame
                last_time       = Some(frame_time);
            } else {
                // This will be the start time for the next keyframe
                last_time = Some(frame_time);
            }
        }

        // The last keyframe has a maximum time to it
        if let Some(start_time) = last_time.take() {
            if start_time != end {
                let start_time  = Self::int_to_time(start_time);
                let end_time    = Self::int_to_time(end);

                result.push(StorageResponse::KeyFrame(start_time, end_time))
            }
        }

        Ok(result)
    }

    ///
    /// Attempts to attach an element to the keyframe nearest the specified time
    ///
    fn attach_element_to_layer(&mut self, layer_id: u64, element_id: i64, when: Duration) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        // Find the nearest keyframe to the requested time
        let when        = Self::time_to_int(when);
        let when        = self.read_previous_key_frame(layer_id, when)?;
        let when        = match when {
            Some(when)  => when,
            None        => { return Ok(vec![StorageResponse::NotFound]); }
        };

        // Write out the attachment
        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO ElementKeyframeAttachment (ElementId, LayerId, TimeMicroseconds) VALUES (?, ?, ?);")?;
        write.execute(&[element_id, layer_id as i64, when])?;

        return Ok(vec![StorageResponse::Updated]);
    }

    ///
    /// Removes an element from any layer it's attached to
    ///
    fn detach_element_from_layer(&mut self, element_id: i64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        // Remove the attachment
        let mut delete   = self.connection.prepare_cached("DELETE FROM ElementKeyframeAttachment WHERE ElementId = ?;")?;
        delete.execute(&[element_id])?;

        return Ok(vec![StorageResponse::Updated]);
    }

    ///
    /// Retrieves the layers and keyframes a particular element is currently attached to
    ///
    fn read_element_attachments(&mut self, element_id: i64) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let mut read    = self.connection.prepare_cached("SELECT LayerId, TimeMicroseconds FROM ElementKeyframeAttachment WHERE ElementId = ?;")?;
        let attachments = read.query_map(&[element_id], |row| Ok((row.get::<_, i64>(0)? as u64, Self::int_to_time(row.get(1)?))))?;

        Ok(vec![StorageResponse::ElementAttachments(element_id, attachments.collect::<Result<Vec<_>, _>>()?)])
    }

    ///
    /// Retrieves the elements attached to a particular key frame
    ///
    fn read_elements_for_key_frame(&mut self, layer_id: u64, when: Duration) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        // Find the nearest keyframe to the requested time
        let when        = Self::time_to_int(when);
        let when        = self.read_previous_key_frame(layer_id, when)?;
        let when        = match when {
            Some(when)  => when,
            None        => { return Ok(vec![]); }
        };

        // Try to read the elements for this keyframe
        let mut read    = self.connection.prepare_cached("
            SELECT Elements.ElementId, Elements.Element FROM Elements
            INNER JOIN ElementKeyframeAttachment ON ElementKeyframeAttachment.ElementId = Elements.ElementId
            WHERE ElementKeyframeAttachment.LayerId = ? AND ElementKeyFrameAttachment.TimeMicroseconds = ?;")?;

        let elements    = read.query_map(&[layer_id as i64, when], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let elements    = elements.map(|element| element.map(|(element_id, element)| StorageResponse::Element(element_id, element)));

        elements.collect()
    }

    ///
    /// Writes a value to the layer cache at a particular time
    ///
    fn write_layer_cache(&mut self, layer_id: u64, when: Duration, cache_type: String, value: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let when        = Self::time_to_int(when);

        let mut write   = self.connection.prepare_cached("INSERT OR REPLACE INTO LayerCache (LayerId, TimeMicroseconds, CacheType, Cache) VALUES (?, ?, ?, ?);")?;
        write.execute(params![layer_id as i64, when, cache_type, value])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Removes a previously cached value
    ///
    fn delete_layer_cache(&mut self, layer_id: u64, when: Duration, cache_type: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        let when        = Self::time_to_int(when);

        let mut write   = self.connection.prepare_cached("DELETE FROM LayerCache WHERE LayerId = ? AND TimeMicroseconds = ? AND CacheType = ?;")?;
        write.execute(params![layer_id as i64, when, cache_type])?;

        Ok(vec![StorageResponse::Updated])
    }

    ///
    /// Reads the value contained in the specified location of the layer cache
    ///
    fn read_layer_cache(&mut self, layer_id: u64, when: Duration, cache_type: String) -> Result<Vec<StorageResponse>, rusqlite::Error> {
        use rusqlite::Error::QueryReturnedNoRows;

        let when        = Self::time_to_int(when);

        let mut read    = self.connection.prepare_cached("SELECT Cache FROM LayerCache WHERE LayerId = ? AND TimeMicroseconds = ? AND CacheType = ?;")?;
        let result      = read.query_row(params![layer_id as i64, when, cache_type], |row| row.get(0));

        match result {
            Ok(cache)                   => Ok(vec![StorageResponse::LayerCache(cache)]),
            Err(QueryReturnedNoRows)    => Ok(vec![StorageResponse::NotFound]),
            Err(other)                  => Err(other)
        }
    }
}
