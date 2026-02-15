use super::canvas::*;
use super::id_cache::*;
use super::super::brush::*;
use super::super::error::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;
use super::super::shape_type::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use rusqlite::*;

use std::collections::{HashMap};
use std::result::{Result};
use std::time::{Duration};

impl SqliteCanvas {
    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_layer(&mut self, layer_id: CanvasLayerId) -> Result<i64, CanvasError> {
        if let Some(cached_id) = self.layer_id_cache.get(&layer_id) {
            Ok(*cached_id)
        } else {
            let idx = self.sqlite.query_one::<i64, _, _>("SELECT LayerId FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0))?;
            self.layer_id_cache.insert(layer_id, idx);
            Ok(idx)
        }
    }

    ///
    /// Returns the time where the frame starts
    ///
    #[inline]
    pub fn layer_frame_time(&mut self, layer_id: CanvasLayerId, when: Duration) -> Result<Duration, CanvasError> {
        let layer_idx = self.index_for_layer(layer_id)?;

        let mut most_recent_time = self.sqlite.prepare_cached("SELECT MAX(Time) FROM LayerFrames WHERE LayerId = ? AND Time <= ?")?;
        let mut most_recent_time = most_recent_time.query_map(params![layer_idx, when.as_nanos() as i64], |row| row.get::<_, Option<i64>>(0))?;
        let most_recent_time     = most_recent_time.next().unwrap_or(Ok(None))?;
        let most_recent_time     = Duration::from_nanos(most_recent_time.unwrap_or(0) as u64);

        Ok(most_recent_time)
    }
}