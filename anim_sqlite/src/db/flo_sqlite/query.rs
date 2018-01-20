use super::*;

use animation::*;

impl FloSqlite {
    ///
    /// Queries a single row in the database
    /// 
    fn query_row<T, F: FnOnce(&Row) -> T>(&mut self, statement: FloStatement, params: &[&ToSql], f: F) -> Result<T> {
        self.flush_pending()?;

        let mut statement = Self::prepare(&self.sqlite, statement)?;
        statement.query_row(params, f)
    }

    ///
    /// Queries and maps some rows
    /// 
    fn query_map<'a, T: 'a, F: FnMut(&Row) -> T>(&mut self, statement: FloStatement, params: &[&ToSql], f: F) -> Result<Box<'a+Iterator<Item=Result<T>>>> {
        self.flush_pending()?;

        // Prepare the statement
        let mut statement = Self::prepare(&self.sqlite, statement)?;

        // Gather the results into a vector (can't keep the map due to lifetime requirements: Rust can't preserve the statement outside of this function)
        let results: Vec<Result<T>> = statement.query_map(params, f)?.collect();

        // Convert into an iterator (into_iter preserves the lifetime of the vec so we don't have the same problem)
        Ok(Box::new(results.into_iter()))
    }
}

impl FloQuery for FloSqlite {
    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<i64> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectLayerId, &[&animation_id, &(assigned_id as i64)], |row| row.get(0))
    }

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64) -> Result<Vec<Duration>> {
        let rows = self.query_map(FloStatement::SelectKeyFrameTimes, &[&layer_id], |row| { Self::from_micros(row.get(0)) })?;
        let rows = rows.map(|row| row.unwrap());

        Ok(rows.collect())
    }

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64)> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectAnimationSize, &[&animation_id], |row| (row.get(0), row.get(1)))
    }

    ///
    /// Returns the assigned layer IDs
    /// 
    fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>> {
        let animation_id = self.animation_id;
        let rows = self.query_map(
            FloStatement::SelectAssignedLayerIds, 
            &[&animation_id],
            |row| {
                let layer_id: i64 = row.get(0);
                layer_id as u64
            })?;

        Ok(rows.filter(|row| row.is_ok()).map(|row| row.unwrap()).collect())
    }

    ///
    /// Retrieves the total number of entries in the edit log
    /// 
    fn query_edit_log_length(&mut self) -> Result<i64> {
        self.query_row(FloStatement::SelectEditLogLength, &[], |row| row.get(0))
    }

    ///
    /// Retrieves a set of values from the edit log
    /// 
    fn query_edit_log_values(&mut self, from_index: i64, to_index: i64) -> Result<Vec<EditLogEntry>> {
        #[inline]
        fn as_id(id_in: Option<i64>) -> Option<u64> {
            match id_in {
                Some(id_in) => Some(id_in as u64),
                None        => None
            }
        }

        #[inline]
        fn as_duration(time_in: Option<i64>) -> Option<Duration> {
            match time_in {
                Some(time_in)   => Some(FloSqlite::from_micros(time_in)),
                None            => None
            }
        }

        self.query_map(FloStatement::SelectEditLogValues, &[&(to_index-from_index), &(from_index)],
            |row| {
                EditLogEntry {
                    edit_id:                row.get(0),
                    edit_type:              EditLogType::AddNewLayer, /* TODO */
                    layer_id:               as_id(row.get(2)),
                    when:                   as_duration(row.get(3)),
                    brush:                  as_id(row.get(5)).and_then(|brush_id| Some((brush_id, DrawingStyleType::Draw))), /* TODO */
                    brush_properties_id:    as_id(row.get(6))
                }
            }).map(|i| i.map(|j| j.unwrap()).collect())
    }

    ///
    /// Queries the size associated with an edit log entry
    /// 
    fn query_edit_log_size(&mut self, edit_id: i64) -> Result<(f64, f64)> {
        unimplemented!()
    }

    ///
    /// Retrieves the raw points associated with a particular edit ID
    /// 
    fn query_edit_log_raw_points(&mut self, edit_id: i64) -> Result<Vec<RawPoint>> {
        unimplemented!()
    }

    ///
    /// Retrieves a colour with the specified ID
    /// 
    fn query_color(&mut self, color_id: i64) -> Result<ColorEntry> {
        unimplemented!()
    }

    ///
    /// Retrieves the brush with the specified ID
    /// 
    fn query_brush(&mut self, brush_id: i64) -> Result<BrushEntry> {
        unimplemented!()
    }

    ///
    /// Retrieves the brush properties with the specified ID
    /// 
    fn query_brush_properties(&mut self, brush_properties_id: i64) -> Result<BrushPropertiesEntry> {
        unimplemented!()
    }
}
