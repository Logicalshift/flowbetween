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

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn order_for_layer(&mut self, layer_id: CanvasLayerId) -> Result<i64, CanvasError> {
        Ok(self.sqlite.query_one::<i64, _, _>("SELECT OrderIdx FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0))?)
    }

    ///
    /// Queries the database for the index of the specified layer
    ///
    #[inline]
    pub fn order_for_layer_in_transaction(transaction: &Transaction<'_>, layer_id: CanvasLayerId) -> Result<i64, CanvasError> {
        Ok(transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0))?)
    }

    ///
    /// Inserts a block of shapes into ShapeLayers at the specified position, shifting existing entries to make room.
    ///
    pub (super) fn insert_shapes_on_layer(transaction: &Transaction<'_>, layer_id: i64, at_order: i64, shape_ids: &[i64], time: i64) -> Result<(), CanvasError> {
        let block_size = shape_ids.len() as i64;

        // Make room for the block
        transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx + ? WHERE LayerId = ? AND OrderIdx >= ?", params![block_size, layer_id, at_order])?;

        // Insert each shape
        let mut insert = transaction.prepare_cached("INSERT INTO ShapeLayers (ShapeId, LayerId, OrderIdx, Time) VALUES (?, ?, ?, ?)")?;
        for (i, shape_id) in shape_ids.iter().enumerate() {
            insert.execute(params![shape_id, layer_id, at_order + i as i64, time])?;
        }

        Ok(())
    }

    ///
    /// Removes a contiguous block of entries from ShapeLayers and compacts the ordering.
    ///
    pub (super) fn remove_shapes_from_layer(transaction: &Transaction<'_>, layer_id: i64, from_order: i64, block_size: i64) -> Result<(), CanvasError> {
        // Delete the block
        transaction.execute("DELETE FROM ShapeLayers WHERE LayerId = ? AND OrderIdx >= ? AND OrderIdx < ?", params![layer_id, from_order, from_order + block_size])?;

        // Compact the ordering
        transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - ? WHERE LayerId = ? AND OrderIdx >= ?", params![block_size, layer_id, from_order + block_size])?;

        Ok(())
    }

    ///
    /// Updates the properties for a layer
    ///
    pub fn set_layer_properties(&mut self, layer_id: CanvasLayerId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
        let layer_idx = self.index_for_layer(layer_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction()?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerIntProperties (LayerId, PropertyId, IntValue) VALUES (?, ?, ?)")?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&layer_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerFloatProperties (LayerId, PropertyId, FloatValue) VALUES (?, ?, ?)")?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&layer_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerBlobProperties (LayerId, PropertyId, BlobValue) VALUES (?, ?, ?)")?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&layer_idx])?;
        }

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Adds a new layer to the canvas
    ///
    pub fn add_layer(&mut self, new_layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>) -> Result<(), CanvasError> {
        let transaction = self.sqlite.transaction()?;

        let new_layer_order = if let Some(before_layer) = before_layer {
            // Add between the existing layers
            let before_order = Self::order_for_layer_in_transaction(&transaction, before_layer)?;
            transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx + 1 WHERE OrderIdx >= ?", [before_order])?;

            before_order
        } else {
            // Add the layer at the end
            let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM Layers", [], |row| row.get(0))?;
            max_order.map(|idx| idx + 1).unwrap_or(0)
        };

        // Add the layer itself
        let new_layer_idx: i64 = transaction.query_one("INSERT INTO Layers(LayerGuid, OrderIdx) VALUES (?, ?) RETURNING LayerId", params![new_layer_id.to_string(), new_layer_order], |row| row.get(0))?;

        transaction.commit()?;

        self.layer_id_cache.insert(new_layer_id, new_layer_idx);

        Ok(())
    }

    ///
    /// Removes an existing layer
    ///
    pub fn remove_layer(&mut self, old_layer_id: CanvasLayerId) -> Result<(), CanvasError> {
        let transaction = self.sqlite.transaction()?;

        let old_layer_order = Self::order_for_layer_in_transaction(&transaction, old_layer_id)?;
        transaction.execute("DELETE FROM Layers WHERE OrderIdx = ?", params![old_layer_order])?;
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx - 1 WHERE OrderIdx >= ?", params![old_layer_order])?;

        transaction.commit()?;

        self.layer_id_cache.remove(&old_layer_id);

        Ok(())
    }

    ///
    /// Adds a frame to a layer at the specified time with the specified length
    ///
    pub fn add_frame(&mut self, frame_layer: CanvasLayerId, when: Duration, _length: Duration) -> Result<(), CanvasError> {
        let layer_idx   = self.index_for_layer(frame_layer)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("INSERT INTO LayerFrames (LayerId, Time) VALUES (?, ?)", params![layer_idx, when_nanos])?;

        Ok(())
    }

    ///
    /// Removes a frame from a layer at the specified time
    ///
    pub fn remove_frame(&mut self, frame_layer: CanvasLayerId, when: Duration) -> Result<(), CanvasError> {
        // TODO: also remove the shapes that exist in this timeframe

        let layer_idx   = self.index_for_layer(frame_layer)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("DELETE FROM LayerFrames WHERE LayerId = ? AND Time = ?", params![layer_idx, when_nanos])?;

        Ok(())
    }
}
