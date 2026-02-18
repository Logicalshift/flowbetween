use super::canvas::*;
use super::super::brush::*;
use super::super::error::*;
use super::super::property::*;
use super::super::shape::*;
use super::super::shape_type::*;

use rusqlite::*;

use std::result::{Result};
use std::time::{Duration};

impl SqliteCanvas {
    ///
    /// Retrieve or create a shape type ID in the database
    ///
    pub (super) fn index_for_shapetype(&mut self, shape_type: ShapeType) -> Result<i64, CanvasError> {
        if let Some(cached_id) = self.shapetype_id_cache.get(&shape_type) {
            // We've encountered this shape type before so we know its ID
            Ok(*cached_id)
        } else {
            // Try to fetch the existing shape type
            let mut query_shapetype = self.sqlite.prepare_cached("SELECT ShapeTypeId FROM ShapeTypes WHERE Name = ?")?;
            if let Some(shapetype_id) = query_shapetype.query_one([shape_type.name()], |row| row.get::<_, i64>(0)).optional()? {
                // Cache it so we don't need to look it up again
                self.shapetype_id_cache.insert(shape_type, shapetype_id);
                self.shapetype_for_id_cache.insert(shapetype_id, shape_type);

                Ok(shapetype_id)
            } else {
                // Create a new shape type ID
                let new_shapetype_id = self.sqlite.query_one("INSERT INTO ShapeTypes (Name) VALUES (?) RETURNING ShapeTypeId", [shape_type.name()], |row| row.get::<_, i64>(0))?;
                self.shapetype_id_cache.insert(shape_type, new_shapetype_id);
                self.shapetype_for_id_cache.insert(new_shapetype_id, shape_type);

                Ok(new_shapetype_id)
            }
        }
    }

    ///
    /// Retrieve the shape type for a database index
    ///
    pub (super) fn shapetype_for_index(&mut self, shapetype_index: i64) -> Result<ShapeType, CanvasError> {
        if let Some(cached_shapetype) = self.shapetype_for_id_cache.get(&shapetype_index) {
            // We've encountered this index before so we know its shape type
            Ok(*cached_shapetype)
        } else {
            // Fetch the shape type name from the database
            let mut query_name  = self.sqlite.prepare_cached("SELECT Name FROM ShapeTypes WHERE ShapeTypeId = ?")?;
            let name            = query_name.query_one([shapetype_index], |row| row.get::<_, String>(0))?;

            // Create the shape type and cache it
            let shape_type = ShapeType::new(&name);
            self.shapetype_id_cache.insert(shape_type, shapetype_index);
            self.shapetype_for_id_cache.insert(shapetype_index, shape_type);

            Ok(shape_type)
        }
    }

    ///
    /// Retrieves the shape type for a shape
    ///
    pub (super) fn shapetype_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<ShapeType, CanvasError> {
        let mut shape_type_query    = self.sqlite.prepare_cached("SELECT ShapeType FROM Shapes WHERE ShapeGuid = ?")?;
        let shape_type              = shape_type_query.query_one(params![shape_id.to_string()], |row| row.get(0))?;
        drop(shape_type_query);

        self.shapetype_for_index(shape_type)
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub (super) fn index_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<i64, rusqlite::Error> {
        if let Some(cached_id) = self.shape_id_cache.get(&shape_id) {
            Ok(*cached_id)
        } else {
            let idx = self.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [shape_id.to_string()], |row| row.get(0))?;
            self.shape_id_cache.insert(shape_id, idx);
            Ok(idx)
        }
    }

    ///
    /// Retrieves the time (in nanoseconds) when a shape appears on the canvas
    ///
    #[inline]
    pub (super) fn time_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<i64, CanvasError> {
        let mut time_query = self.sqlite.prepare_cached("
            SELECT      sl.Time 
            FROM        ShapeLayers sl 
            INNER JOIN  Shapes s ON s.ShapeId = sl.ShapeId 
            WHERE       s.ShapeGuid = ?")?;

        let mut time    = time_query.query_map([shape_id.to_string()], |row| row.get(0))?;
        let time        = time.next().unwrap_or(Ok(0i64))?;

        Ok(time)
    }

    ///
    /// Collects all descendents of a shape in depth-first pre-order (does not include the shape itself).
    ///
    #[inline]
    fn all_descendents_for_shape(transaction: &Transaction<'_>, shape_idx: i64) -> Result<Vec<i64>, CanvasError> {
        let mut result = Vec::new();
        Self::collect_shape_dependents(transaction, shape_idx, &mut result)?;
        Ok(result)
    }

    ///
    /// Returns the shape IDs that have the specified brush attached
    ///
    pub fn shapes_with_brush(&self, brush_id: CanvasBrushId) -> Result<Vec<CanvasShapeId>, CanvasError> {
        let mut query_shapes = self.sqlite.prepare_cached("SELECT s.ShapeGuid FROM ShapeBrushes sb JOIN Brushes b ON sb.BrushId = b.BrushId JOIN Shapes s ON sb.ShapeId = s.ShapeId WHERE b.BrushGuid = ?")?;

        let mut shapes = vec![];
        for row in query_shapes.query_map(params![brush_id.to_string()], |row| Ok(CanvasShapeId::from_string(&row.get::<_, String>(0)?)))? {
            shapes.push(row?);
        }

        Ok(shapes)
    }

    ///
    /// Recurses through the descendents of a shape
    ///
    fn collect_shape_dependents(transaction: &Transaction<'_>, parent_idx: i64, result: &mut Vec<i64>) -> Result<(), CanvasError> {
        // Using a recursive Rust function because SQL CTEs don't guarantee depth-first ordering.
        let mut stmt = transaction.prepare_cached("SELECT ShapeId FROM ShapeGroups WHERE ParentShapeId = ? ORDER BY OrderIdx ASC")?;
        let children: Vec<i64> = stmt.query_map(params![parent_idx], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        for child in children {
            result.push(child);
            Self::collect_shape_dependents(transaction, child, result)?;
        }

        Ok(())
    }

    ///
    /// Updates the properties for a shape
    ///
    pub fn set_shape_properties(&mut self, shape_id: CanvasShapeId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
        let shape_idx = self.index_for_shape(shape_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction()?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeIntProperties (ShapeId, PropertyId, IntValue) VALUES (?, ?, ?)")?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&shape_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeFloatProperties (ShapeId, PropertyId, FloatValue) VALUES (?, ?, ?)")?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&shape_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeBlobProperties (ShapeId, PropertyId, BlobValue) VALUES (?, ?, ?)")?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&shape_idx])?;
        }

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Encodes a canvas shape as a (shape_type, shape_data) pair for database storage
    ///
    /// Shape types are defined by the CANVAS_*_V1_TYPE constants
    ///
    fn encode_shape(shape: &CanvasShape) -> Result<(i64, Vec<u8>), CanvasError> {
        match shape {
            CanvasShape::Path(path)         => Ok((CANVAS_PATH_V1_TYPE, postcard::to_allocvec(path)?)),
            CanvasShape::Group              => Ok((CANVAS_GROUP_V1_TYPE, vec![])),
            CanvasShape::Rectangle(rect)    => Ok((CANVAS_RECTANGLE_V1_TYPE, postcard::to_allocvec(rect)?)),
            CanvasShape::Ellipse(ellipse)   => Ok((CANVAS_ELLIPSE_V1_TYPE, postcard::to_allocvec(ellipse)?)),
            CanvasShape::Polygon(polygon)   => Ok((CANVAS_POLYGON_V1_TYPE, postcard::to_allocvec(polygon)?)),
        }
    }

    ///
    /// Adds a new shape to the canvas, or replaces the definition if the shape ID is already in use
    ///
    pub fn add_shape(&mut self, shape_id: CanvasShapeId, shape_type: ShapeType, shape: CanvasShape) -> Result<(), CanvasError> {
        let shape_type_idx                  = self.index_for_shapetype(shape_type)?;
        let (shape_data_type, shape_data)   = Self::encode_shape(&shape)?;

        if let Some(existing_idx) = self.index_for_shape(shape_id).optional()? {
            // Replace the existing shape definition in place
            let mut update_existing = self.sqlite.prepare_cached("UPDATE Shapes SET ShapeType = ?, ShapeDataType = ?, ShapeData = ? WHERE ShapeId = ?")?;
            update_existing.execute(params![shape_type_idx, shape_data_type, shape_data, existing_idx])?;
        } else {
            // Insert a new shape with a generated ShapeId
            let mut insert_new  = self.sqlite.prepare_cached("INSERT INTO Shapes (ShapeId, ShapeGuid, ShapeType, ShapeDataType, ShapeData) VALUES (?, ?, ?, ?, ?)")?;
            let next_id: i64    = if let Some(cached_id) = self.next_shape_id {
                cached_id
            } else {
                let mut get_max_id = self.sqlite.prepare_cached("SELECT COALESCE(MAX(ShapeId), 0) + 1 FROM Shapes")?;
                get_max_id.query_one([], |row| row.get(0))?
            };

            insert_new.execute(params![next_id, shape_id.to_string(), shape_type_idx, shape_data_type, shape_data])?;

            // Store the shape ID in the cache so we can look it up faster for things like setting properties
            self.shape_id_cache.insert(shape_id, next_id);
            self.next_shape_id = Some(next_id + 1);
        }

        Ok(())
    }

    ///
    /// Removes a shape and all its associations from the canvas
    ///
    pub fn remove_shape(&mut self, shape_id: CanvasShapeId) -> Result<(), CanvasError> {
        let shape_idx = self.index_for_shape(shape_id)?;

        let transaction = self.sqlite.transaction()?;

        // Query parent info for order compaction before the cascading delete
        let layer_info = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).ok();
        let group_info = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).ok();

        // Collect descendents for block-size compaction and deletion
        let descendents = Self::all_descendents_for_shape(&transaction, shape_idx)?;
        let block_size  = 1 + descendents.len() as i64;

        // Delete all descendents from Shapes (CASCADE handles their ShapeLayers, ShapeGroups, properties)
        for desc_id in &descendents {
            transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![desc_id])?;
        }

        // Delete the shape: CASCADE handles ShapeLayers, ShapeGroups, ShapeBrushes, and properties
        transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![shape_idx])?;

        // Compact ordering in the parent layer by block_size (shape + all descendents were contiguous)
        if let Some((layer_id, order_idx)) = layer_info {
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - ? WHERE LayerId = ? AND OrderIdx > ?", params![block_size, layer_id, order_idx])?;
        }

        // Compact ordering in the parent group by 1 (only the shape itself was a direct child)
        if let Some((parent_id, order_idx)) = group_info {
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx])?;
        }

        transaction.commit()?;

        // Invalidate caches for the removed shape and its descendents
        self.shape_id_cache.remove(&shape_id);
        if !descendents.is_empty() {
            self.shape_id_cache.retain(|_, idx| !descendents.contains(idx));
        }
        self.next_shape_id = None;

        Ok(())
    }

    ///
    /// Replaces the definition of an existing shape, preserving its parent, properties, and brushes
    ///
    pub fn set_shape_definition(&mut self, shape_id: CanvasShapeId, shape: CanvasShape) -> Result<(), CanvasError> {
        let shape_idx                   = self.index_for_shape(shape_id)?;
        let (shape_type, shape_data)    = Self::encode_shape(&shape)?;

        self.sqlite.execute("UPDATE Shapes SET ShapeDataType = ?, ShapeData = ? WHERE ShapeId = ?", params![shape_type, shape_data, shape_idx])?;

        Ok(())
    }

    ///
    /// Sets the time when a shape should appear on its layer
    ///
    pub fn set_shape_time(&mut self, shape_id: CanvasShapeId, when: Duration) -> Result<(), CanvasError> {
        let shape_idx   = self.index_for_shape(shape_id)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("UPDATE ShapeLayers SET Time = ? WHERE ShapeId = ?", params![when_nanos, shape_idx])?;

        Ok(())
    }

    ///
    /// Reorders a shape within its current parent (layer or group)
    ///
    pub fn reorder_shape(&mut self, shape_id: CanvasShapeId, before_shape: Option<CanvasShapeId>) -> Result<(), CanvasError> {
        let shape_idx           = self.index_for_shape(shape_id)?;
        let before_shape_idx    = before_shape.map(|bs| self.index_for_shape(bs)).transpose()?;

        let transaction = self.sqlite.transaction()?;

        // Check if shape is in a group first
        if let Some((parent_id, original_order)) = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).optional()? {
            // Read the existing order of the shape within a group
            let before_order = if let Some(before_idx) = before_shape_idx {
                transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeGroups WHERE ShapeId = ? AND ParentShapeId = ?", params![before_idx, parent_id], |row| row.get(0))?
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_id], |row| row.get(0))?;
                max_order.map(|idx| idx + 1).unwrap_or(0)
            };

            // Reorder within ShapeGroups
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, original_order])?;
            let before_order = if before_order > original_order { before_order - 1 } else { before_order };

            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx + 1 WHERE ParentShapeId = ? AND OrderIdx >= ?", params![parent_id, before_order])?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = ? WHERE ShapeId = ?", params![before_order, shape_idx])?;

            // Rebuild the parent group's ShapeLayers descendents to reflect the new ordering
            if let Some((layer_id, parent_order_idx, parent_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![parent_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))).optional()? {
                // Collect new depth-first order (ShapeGroups already reordered)
                let new_descendents = Self::all_descendents_for_shape(&transaction, parent_id)?;
                let desc_count      = new_descendents.len() as i64;

                // Remove old descendent entries from ShapeLayers
                Self::remove_shapes_from_layer(&transaction, layer_id, parent_order_idx + 1, desc_count)?;

                // Re-insert in new depth-first order, preserving the parent group's time
                Self::insert_shapes_on_layer(&transaction, layer_id, parent_order_idx + 1, &new_descendents, parent_time)?;
            }

            transaction.commit()?;
            return Ok(());
        }

        // Check if shape is directly on a layer (not in a group)
        if let Some((layer_id, original_order, original_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))).optional()? {
            // Collect descendents for block movement
            let descendents = Self::all_descendents_for_shape(&transaction, shape_idx)?;
            let block_size  = 1 + descendents.len() as i64;

            // Build the block
            let mut block = vec![shape_idx];
            block.extend(&descendents);

            // Remove the block from its current position
            Self::remove_shapes_from_layer(&transaction, layer_id, original_order, block_size)?;

            // Query the target position and time (after removal, so positions are up to date)
            let (before_order, new_time) = if let Some(before_idx) = before_shape_idx {
                let (order, time) = transaction.query_one("SELECT OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ? AND LayerId = ?", params![before_idx, layer_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
                (order, time)
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_id], |row| row.get(0))?;
                (max_order.map(|idx| idx + 1).unwrap_or(0), original_time)
            };

            // Re-insert the block at the new position with the target time
            Self::insert_shapes_on_layer(&transaction, layer_id, before_order, &block, new_time)?;

            transaction.commit()?;
            return Ok(());
        }

        // Shape has no parent, cannot reorder
        Err(CanvasError::ShapeHasNoParent(shape_id))
    }

    ///
    /// Sets the parent of a shape, placing it as the topmost (last) shape in the new parent
    ///
    pub fn set_shape_parent(&mut self, shape_id: CanvasShapeId, parent: CanvasShapeParent) -> Result<(), CanvasError> {
        let shape_idx = self.index_for_shape(shape_id)?;

        // Look up the new parent index before starting the transaction
        let new_layer_idx;
        let new_parent_shape_idx;
        let when_nanos;

        match &parent {
            CanvasShapeParent::Layer(layer_id, when) => {
                // Specified in the request
                new_layer_idx           = Some(self.index_for_layer(*layer_id)?);
                new_parent_shape_idx    = None;
                when_nanos              = when.as_nanos() as i64;
            }

            CanvasShapeParent::Shape(parent_shape_id) => {
                new_layer_idx           = Some(self.time_for_shape(*parent_shape_id)?);
                new_parent_shape_idx    = Some(self.index_for_shape(*parent_shape_id)?);
                when_nanos              = 0;
            }

            CanvasShapeParent::None => {
                // Unparented shape
                new_layer_idx           = None;
                new_parent_shape_idx    = None;
                when_nanos              = 0;
            }
        }

        // Perform the update itself in a transaction
        let transaction = self.sqlite.transaction()?;

        // Collect descendents for block operations (the shape and its descendents move together)
        let descendents = Self::all_descendents_for_shape(&transaction, shape_idx)?;
        let block_size  = 1 + descendents.len() as i64;

        // Remove from ShapeLayers (covers both direct layer parent and group-via-layer)
        if let Some((layer_id, order_idx)) = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).optional()? {
            Self::remove_shapes_from_layer(&transaction, layer_id, order_idx, block_size)?;
        }

        // Remove from any existing group parent
        if let Some((parent_id, order_idx)) = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).optional()? {
            transaction.execute("DELETE FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx])?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx])?;
        }

        // Build the block of shapes to insert (shape + descendents in depth-first order)
        let mut block = vec![shape_idx];
        block.extend(&descendents);

        // Add to the new parent at the end
        match parent {
            CanvasShapeParent::None => {
                // Shape is detached, nothing more to do
            }

            CanvasShapeParent::Layer(..) => {
                let layer_idx  = new_layer_idx.unwrap();
                let next_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_idx], |row| row.get(0))?;
                let next_order = next_order.map(|idx| idx + 1).unwrap_or(0);

                Self::insert_shapes_on_layer(&transaction, layer_idx, next_order, &block, when_nanos)?;
            }

            CanvasShapeParent::Shape(_) => {
                let parent_shape_idx = new_parent_shape_idx.unwrap();

                // Count the parent group's current descendents before inserting (for finding the insertion point in ShapeLayers)
                let parent_old_descendents = Self::all_descendents_for_shape(&transaction, parent_shape_idx)?;
                let parent_old_block_size  = 1 + parent_old_descendents.len() as i64;

                // Add to ShapeGroups at the end
                let next_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_shape_idx], |row| row.get(0))?;
                let next_order = next_order.map(|idx| idx + 1).unwrap_or(0);

                transaction.execute("INSERT INTO ShapeGroups (ShapeId, ParentShapeId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, parent_shape_idx, next_order])?;

                // Also add to ShapeLayers if the parent group is on a layer
                if let Some((layer_id, parent_sl_order, parent_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![parent_shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))).optional()? {
                    let insert_at = parent_sl_order + parent_old_block_size;
                    Self::insert_shapes_on_layer(&transaction, layer_id, insert_at, &block, parent_time)?;
                }
            }
        }

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Adds brush associations to a shape
    ///
    pub fn add_shape_brushes(&mut self, shape_id: CanvasShapeId, brush_ids: Vec<CanvasBrushId>) -> Result<(), CanvasError> {
        let shape_idx               = self.index_for_shape(shape_id)?;
        let brush_indices: Vec<i64> = brush_ids.iter()
            .map(|brush_id| self.index_for_brush(*brush_id))
            .collect::<Result<Vec<_>, _>>()?;

        let transaction = self.sqlite.transaction()?;

        let mut next_order: i64 = transaction.query_one("SELECT COALESCE(MAX(OrderIdx), -1) + 1 FROM ShapeBrushes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0))?;

        for brush_idx in brush_indices {
            transaction.execute("INSERT INTO ShapeBrushes (ShapeId, BrushId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, brush_idx, next_order])?;
            next_order += 1;
        }

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Removes brush associations from a shape
    ///
    pub fn remove_shape_brushes(&mut self, shape_id: CanvasShapeId, brush_ids: Vec<CanvasBrushId>) -> Result<(), CanvasError> {
        let shape_idx               = self.index_for_shape(shape_id)?;
        let brush_indices: Vec<i64> = brush_ids.iter()
            .map(|brush_id| self.index_for_brush(*brush_id))
            .collect::<Result<Vec<_>, _>>()?;

        let transaction = self.sqlite.transaction()?;

        for brush_idx in brush_indices {
            if let Some(order_idx) = transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeBrushes WHERE ShapeId = ? AND BrushId = ?", params![shape_idx, brush_idx], |row| row.get(0)).optional()? {
                transaction.execute("DELETE FROM ShapeBrushes WHERE ShapeId = ? AND BrushId = ?", params![shape_idx, brush_idx])?;
                transaction.execute("UPDATE ShapeBrushes SET OrderIdx = OrderIdx - 1 WHERE ShapeId = ? AND OrderIdx > ?", params![shape_idx, order_idx])?;
            }
        }

        transaction.commit()?;

        Ok(())
    }
}
