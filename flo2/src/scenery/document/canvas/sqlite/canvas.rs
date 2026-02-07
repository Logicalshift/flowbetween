use super::super::brush::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use rusqlite::*;

use std::collections::{HashMap};
use std::result::{Result};

/// Definition for the canvas sqlite storage
pub (super) static SCHEMA: &'static str = include_str!("canvas.sql");

///
/// Storage for the sqlite canvas
///
pub struct SqliteCanvas {
    /// Connection to the sqlite database
    pub (super) sqlite: Connection,

    /// Cache of the known property IDs
    pub (super) property_id_cache: HashMap<CanvasPropertyId, i64>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Result<Self, ()> {
        sqlite.execute_batch("PRAGMA foreign_keys = ON").map_err(|_| ())?;

        Ok(Self {
            sqlite:             sqlite,
            property_id_cache:  HashMap::new(),
        })
    }

    ///
    /// Initialises the canvas in this object
    ///
    pub fn initialise(&self) -> Result<(), ()> {
        self.sqlite.execute_batch(SCHEMA).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Creates a new SQLite canvas in memory
    ///
    pub fn new_in_memory() -> Result<Self, ()> {
        let sqlite  = Connection::open_in_memory().map_err(|_| ())?;
        let canvas  = Self::with_connection(sqlite)?;
        canvas.initialise()?;

        Ok(canvas)
    }

    ///
    /// Sets the properties for a property target
    ///
    pub fn set_properties(&mut self, target: CanvasPropertyTarget, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), ()> {
        match target {
            CanvasPropertyTarget::Document          => self.set_document_properties(properties),
            CanvasPropertyTarget::Layer(layer_id)   => self.set_layer_properties(layer_id, properties),
            CanvasPropertyTarget::Brush(brush_id)   => self.set_brush_properties(brush_id, properties),
            CanvasPropertyTarget::Shape(shape_id)   => self.set_shape_properties(shape_id, properties),
        }
    }

    ///
    /// Retrieve or create a property ID in the database
    ///
    pub (super) fn index_for_property(&mut self, canvas_property_id: CanvasPropertyId) -> Result<i64, ()> {
        if let Some(cached_id) = self.property_id_cache.get(&canvas_property_id) {
            // We've encountered this property before so we know its ID
            Ok(*cached_id)
        } else {
            // Try to fetch the existing property
            let mut query_property = self.sqlite.prepare_cached("SELECT PropertyId FROM Properties WHERE Name = ?").map_err(|_| ())?;
            if let Ok(property_id) = query_property.query_one([canvas_property_id.name()], |row| row.get::<_, i64>(0)) {
                // Cache it so we don't need to look it up again
                self.property_id_cache.insert(canvas_property_id, property_id);

                Ok(property_id)
            } else {
                // Create a new property ID
                let new_property_id = self.sqlite.query_one("INSERT INTO Properties (Name) VALUES (?) RETURNING PropertyId", [canvas_property_id.name()], |row| row.get::<_, i64>(0)).map_err(|_| ())?;
                self.property_id_cache.insert(canvas_property_id, new_property_id);

                Ok(new_property_id)
            }
        }
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_layer(&mut self, layer_id: CanvasLayerId) -> Result<i64, ()> {
        self.sqlite.query_one::<i64, _, _>("SELECT LayerId FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).map_err(|_| ())
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<i64, ()> {
        self.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [shape_id.to_string()], |row| row.get(0)).map_err(|_| ())
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_brush(&mut self, brush_id: CanvasBrushId) -> Result<i64, ()> {
        self.sqlite.query_one::<i64, _, _>("SELECT BrushId FROM Brushes WHERE BrushGuid = ?", [brush_id.to_string()], |row| row.get(0)).map_err(|_| ())
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn order_for_layer(&mut self, layer_id: CanvasLayerId) -> Result<i64, ()> {
        self.sqlite.query_one::<i64, _, _>("SELECT OrderIdx FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).map_err(|_| ())
    }

    ///
    /// Queries the database for the index of the specified layer
    ///
    #[inline]
    pub fn order_for_layer_in_transaction(transaction: &Transaction<'_>, layer_id: CanvasLayerId) -> Result<i64, ()> {
        transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).map_err(|_| ())
    }

    ///
    /// Sets values in a properties table. Property values are appended to the supplied default parameters
    ///
    #[inline]
    fn set_sql_properties<'a>(properties: impl Iterator<Item=(i64, &'a dyn ToSql)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), ()> {
        for (property_idx, property) in properties {
            // Add the property ID and value to the parameters
            let mut params = other_params.clone();
            params.extend(params![property_idx, property]);

            let params: &[&dyn ToSql] = &params;

            // Run the query
            command.execute(params).map_err(|_| ())?;
        }

        Ok(())
    }

    ///
    /// Sets any int properties found in the specified properties array. Property values are appended to the supplied default parameters
    ///
    fn set_int_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), ()> {
        // Only set the int properties that are requested
        let int_properties = properties.iter()
            .filter_map::<(_, &dyn ToSql), _>(|(property_idx, property)| {
                if let CanvasProperty::Int(val) = property {
                    Some((*property_idx, val))
                } else {
                    None
                }
            });


        // Set each of the int properties
        Self::set_sql_properties(int_properties, command, other_params)?;

        Ok(())
    }

    ///
    /// Sets any float properties found in the specified properties array. Property values are appended to the supplied default parameters
    ///
    fn set_float_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), ()> {
        // Only set the float properties that are requested
        let float_properties = properties.iter()
            .filter_map::<(_, &dyn ToSql), _>(|(property_idx, property)| {
                if let CanvasProperty::Float(val) = property {
                    Some((*property_idx, val))
                } else {
                    None
                }
            });

        Self::set_sql_properties(float_properties, command, other_params)?;

        Ok(())
    }

    ///
    /// Sets any blob properties found in the specified properties array. Property values are appended to the supplied default parameters
    ///
    fn set_blob_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), ()> {
        // Only set the blob properties that are requested
        let blob_properties = properties.iter()
            .filter_map(|(property_idx, property)| {
                match property {
                    CanvasProperty::Int(_)      | 
                    CanvasProperty::Float(_)    => None,
                    property                    => Some(postcard::to_allocvec(property).map(|val| (*property_idx, val)).map_err(|_| ()))
                }
            })
            .collect::<Result<Vec<_>, ()>>()?;

        // Need references to the blobs we've built up
        let blob_properties = blob_properties.iter()
            .map::<(_, &dyn ToSql), _>(|(idx, prop)| (*idx, prop));

        Self::set_sql_properties(blob_properties, command, other_params)?;

        Ok(())
    }

    ///
    /// Updates the properties for a document
    ///
    pub fn set_document_properties(&mut self, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), ()> {
        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentIntProperties (PropertyId, IntValue) VALUES (?, ?)").map_err(|_| ())?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentFloatProperties (PropertyId, FloatValue) VALUES (?, ?)").map_err(|_| ())?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentBlobProperties (PropertyId, BlobValue) VALUES (?, ?)").map_err(|_| ())?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![])?;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Updates the properties for a layer
    ///
    pub fn set_layer_properties(&mut self, layer_id: CanvasLayerId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), ()> {
        let layer_idx = self.index_for_layer(layer_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerIntProperties (LayerId, PropertyId, IntValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&layer_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerFloatProperties (LayerId, PropertyId, FloatValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&layer_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO LayerBlobProperties (LayerId, PropertyId, BlobValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&layer_idx])?;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Updates the properties for a shape
    ///
    pub fn set_shape_properties(&mut self, shape_id: CanvasShapeId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), ()> {
        let shape_idx = self.index_for_shape(shape_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeIntProperties (ShapeId, PropertyId, IntValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&shape_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeFloatProperties (ShapeId, PropertyId, FloatValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&shape_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO ShapeBlobProperties (ShapeId, PropertyId, BlobValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&shape_idx])?;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Updates the properties for a brush
    ///
    pub fn set_brush_properties(&mut self, brush_id: CanvasBrushId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), ()> {
        let brush_idx = self.index_for_brush(brush_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushIntProperties (BrushId, PropertyId, IntValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushFloatProperties (BrushId, PropertyId, FloatValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushBlobProperties (BrushId, PropertyId, BlobValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&brush_idx])?;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Sends a query response stored in a Vec<()>
    ///
    pub async fn send_vec_query_response(&mut self, target: StreamTarget, context: &SceneContext, generate_response: impl FnOnce(&mut SqliteCanvas, &mut Vec<VectorResponse>) -> Result<(), ()>) -> Result<(), ()> {
        // Connect to the query response target
        let mut target      = context.send(target).map_err(|_| ())?;
        let mut response    = vec![];

        // Generate the values to send
        generate_response(self, &mut response)?;

        // Send the query response message
        target.send(QueryResponse::with_iterator(response)).await.map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Adds a new layer to the canvas
    ///
    pub fn add_layer(&mut self, new_layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>) -> Result<(), ()> {
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        let new_layer_order = if let Some(before_layer) = before_layer {
            // Add between the existing layers
            let before_order = Self::order_for_layer_in_transaction(&transaction, before_layer)?;
            transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx + 1 WHERE OrderIdx >= ?", [before_order]).map_err(|_| ())?;

            before_order
        } else {
            // Add the layer at the end
            let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM Layers", [], |row| row.get(0)).map_err(|_| ())?;
            max_order.map(|idx| idx + 1).unwrap_or(0)
        };

        // Add the layer itself
        transaction.execute("INSERT INTO Layers(LayerGuid, OrderIdx) VALUES (?, ?)", params![new_layer_id.to_string(), new_layer_order]).map_err(|_| ())?;

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Removes an existing layer
    ///
    pub fn remove_layer(&mut self, old_layer_id: CanvasLayerId) -> Result<(), ()> {
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        let old_layer_order = Self::order_for_layer_in_transaction(&transaction, old_layer_id)?;
        transaction.execute("DELETE FROM Layers WHERE OrderIdx = ?", params![old_layer_order]).map_err(|_| ())?;
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx - 1 WHERE OrderIdx >= ?", params![old_layer_order]).map_err(|_| ())?;

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Changes the ordering of a layer
    ///
    pub fn reorder_layer(&mut self, layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>) -> Result<(), ()> {
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Work out the layer indexes where we want to add the new layer and the 
        let original_layer_order  = Self::order_for_layer_in_transaction(&transaction, layer_id)?;
        let before_layer_order    = if let Some(before_layer) = before_layer {
            Self::order_for_layer_in_transaction(&transaction, before_layer)?            
        } else {
            let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM Layers", [], |row| row.get(0)).map_err(|_| ())?;
            max_order.map(|idx| idx + 1).unwrap_or(0)
        };

        // Move the layers after the original layer
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx - 1 WHERE OrderIdx > ?", params![original_layer_order]).map_err(|_| ())?;
        let before_layer_order = if before_layer_order > original_layer_order {
            before_layer_order-1
        } else {
            before_layer_order
        };

        // Move the layers after the before layer index
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx + 1 WHERE OrderIdx >= ?", params![before_layer_order]).map_err(|_| ())?;

        // Move the re-ordered layer to its new position
        transaction.execute("UPDATE Layers SET OrderIdx = ? WHERE LayerGuid = ?", params![before_layer_order, layer_id.to_string()]).map_err(|_| ())?;

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Encodes a canvas shape as a (shape_type, shape_data) pair for database storage
    ///
    /// Shape types are defined by the CANVAS_*_V1_TYPE constants
    ///
    fn encode_shape(shape: &CanvasShape) -> Result<(i64, Vec<u8>), ()> {
        match shape {
            CanvasShape::Path(path)         => Ok((CANVAS_PATH_V1_TYPE, postcard::to_allocvec(path).map_err(|_| ())?)),
            CanvasShape::Group              => Ok((CANVAS_GROUP_V1_TYPE, vec![])),
            CanvasShape::Rectangle(rect)    => Ok((CANVAS_RECTANGLE_V1_TYPE, postcard::to_allocvec(rect).map_err(|_| ())?)),
            CanvasShape::Ellipse(ellipse)   => Ok((CANVAS_ELLIPSE_V1_TYPE, postcard::to_allocvec(ellipse).map_err(|_| ())?)),
            CanvasShape::Polygon(polygon)   => Ok((CANVAS_POLYGON_V1_TYPE, postcard::to_allocvec(polygon).map_err(|_| ())?)),
        }
    }

    ///
    /// Adds a new shape to the canvas, or replaces the definition if the shape ID is already in use
    ///
    pub fn add_shape(&mut self, shape_id: CanvasShapeId, shape: CanvasShape) -> Result<(), ()> {
        let (shape_type, shape_data) = Self::encode_shape(&shape)?;

        if let Ok(existing_idx) = self.index_for_shape(shape_id) {
            // Replace the existing shape definition in place
            self.sqlite.execute("UPDATE Shapes SET ShapeType = ?, ShapeData = ? WHERE ShapeId = ?", params![shape_type, shape_data, existing_idx]).map_err(|_| ())?;
        } else {
            // Insert a new shape with a generated ShapeId
            let next_id: i64 = self.sqlite.query_one("SELECT COALESCE(MAX(ShapeId), 0) + 1 FROM Shapes", [], |row| row.get(0)).map_err(|_| ())?;

            self.sqlite.execute("INSERT INTO Shapes (ShapeId, ShapeGuid, ShapeType, ShapeData) VALUES (?, ?, ?, ?)", params![next_id, shape_id.to_string(), shape_type, shape_data]).map_err(|_| ())?;
        }

        Ok(())
    }

    ///
    /// Removes a shape and all its associations from the canvas
    ///
    pub fn remove_shape(&mut self, shape_id: CanvasShapeId) -> Result<(), ()> {
        let shape_idx = self.index_for_shape(shape_id)?;

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Query parent info for order compaction before the cascading delete
        let layer_info = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).ok();
        let group_info = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).ok();

        // Recursively delete all shapes grouped under this shape (and their descendants)
        {
            let mut stmt = transaction.prepare_cached(
                "WITH RECURSIVE descendants(ShapeId) AS (
                    SELECT ShapeId FROM ShapeGroups WHERE ParentShapeId = ?1
                    UNION ALL
                    SELECT sg.ShapeId FROM ShapeGroups sg JOIN descendants d ON sg.ParentShapeId = d.ShapeId
                )
                SELECT ShapeId FROM descendants"
            ).map_err(|_| ())?;
            let descendant_ids: Vec<i64> = stmt.query_map(params![shape_idx], |row| row.get(0))
                .map_err(|_| ())?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            for desc_id in descendant_ids {
                transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![desc_id]).map_err(|_| ())?;
            }
        }

        // Delete the shape: CASCADE handles ShapeLayers, ShapeGroups, ShapeBrushes, and properties
        transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;

        // Compact ordering in the parent layer
        if let Some((layer_id, order_idx)) = layer_info {
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - 1 WHERE LayerId = ? AND OrderIdx > ?", params![layer_id, order_idx]).map_err(|_| ())?;
        }

        // Compact ordering in the parent group
        if let Some((parent_id, order_idx)) = group_info {
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx]).map_err(|_| ())?;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Replaces the definition of an existing shape, preserving its parent, properties, and brushes
    ///
    pub fn set_shape_definition(&mut self, shape_id: CanvasShapeId, shape: CanvasShape) -> Result<(), ()> {
        let shape_idx                   = self.index_for_shape(shape_id)?;
        let (shape_type, shape_data)    = Self::encode_shape(&shape)?;

        self.sqlite.execute("UPDATE Shapes SET ShapeType = ?, ShapeData = ? WHERE ShapeId = ?", params![shape_type, shape_data, shape_idx]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Reorders a shape within its current parent (layer or group)
    ///
    pub fn reorder_shape(&mut self, shape_id: CanvasShapeId, before_shape: Option<CanvasShapeId>) -> Result<(), ()> {
        let shape_idx           = self.index_for_shape(shape_id)?;
        let before_shape_idx    = before_shape.map(|bs| self.index_for_shape(bs)).transpose()?;

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Check if shape is on a layer
        if let Ok((layer_id, original_order)) = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            let before_order = if let Some(before_idx) = before_shape_idx {
                transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeLayers WHERE ShapeId = ? AND LayerId = ?", params![before_idx, layer_id], |row| row.get(0)).map_err(|_| ())?
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_id], |row| row.get(0)).map_err(|_| ())?;
                max_order.map(|idx| idx + 1).unwrap_or(0)
            };

            // Remove from original position
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - 1 WHERE LayerId = ? AND OrderIdx > ?", params![layer_id, original_order]).map_err(|_| ())?;
            let before_order = if before_order > original_order { before_order - 1 } else { before_order };

            // Make space at the new position
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx + 1 WHERE LayerId = ? AND OrderIdx >= ?", params![layer_id, before_order]).map_err(|_| ())?;

            // Move to the new position
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = ? WHERE ShapeId = ? AND LayerId = ?", params![before_order, shape_idx, layer_id]).map_err(|_| ())?;

            transaction.commit().map_err(|_| ())?;
            return Ok(());
        }

        // Check if shape is in a group
        if let Ok((parent_id, original_order)) = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            let before_order = if let Some(before_idx) = before_shape_idx {
                transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeGroups WHERE ShapeId = ? AND ParentShapeId = ?", params![before_idx, parent_id], |row| row.get(0)).map_err(|_| ())?
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_id], |row| row.get(0)).map_err(|_| ())?;
                max_order.map(|idx| idx + 1).unwrap_or(0)
            };

            // Remove from original position
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, original_order]).map_err(|_| ())?;
            let before_order = if before_order > original_order { before_order - 1 } else { before_order };

            // Make space at the new position
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx + 1 WHERE ParentShapeId = ? AND OrderIdx >= ?", params![parent_id, before_order]).map_err(|_| ())?;

            // Move to the new position
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = ? WHERE ShapeId = ?", params![before_order, shape_idx]).map_err(|_| ())?;

            transaction.commit().map_err(|_| ())?;
            return Ok(());
        }

        // Shape has no parent, cannot reorder
        Err(())
    }

    ///
    /// Sets the parent of a shape, placing it as the topmost (last) shape in the new parent
    ///
    pub fn set_shape_parent(&mut self, shape_id: CanvasShapeId, parent: CanvasShapeParent) -> Result<(), ()> {
        let shape_idx = self.index_for_shape(shape_id)?;

        // Look up the new parent index before starting the transaction
        let new_layer_idx = if let CanvasShapeParent::Layer(layer_id) = &parent {
            Some(self.index_for_layer(*layer_id)?)
        } else {
            None
        };
        let new_parent_shape_idx = if let CanvasShapeParent::Shape(parent_id) = &parent {
            Some(self.index_for_shape(*parent_id)?)
        } else {
            None
        };

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Remove from any existing layer parent
        if let Ok((layer_id, order_idx)) = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            transaction.execute("DELETE FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - 1 WHERE LayerId = ? AND OrderIdx > ?", params![layer_id, order_idx]).map_err(|_| ())?;
        }

        // Remove from any existing group parent
        if let Ok((parent_id, order_idx)) = transaction.query_one(
            "SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?",
            params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ) {
            transaction.execute("DELETE FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx]).map_err(|_| ())?;
        }

        // Add to the new parent at the end
        match parent {
            CanvasShapeParent::None => {
                // Shape is detached, nothing more to do
            }

            CanvasShapeParent::Layer(_) => {
                let layer_idx  = new_layer_idx.unwrap();
                let next_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).map_err(|_| ())?;
                let next_order = next_order.map(|idx| idx + 1).unwrap_or(0);

                transaction.execute("INSERT INTO ShapeLayers (ShapeId, LayerId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, layer_idx, next_order]).map_err(|_| ())?;
            }

            CanvasShapeParent::Shape(_) => {
                let parent_shape_idx = new_parent_shape_idx.unwrap();
                let next_order       = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_shape_idx], |row| row.get(0)).map_err(|_| ())?;
                let next_order       = next_order.map(|idx| idx + 1).unwrap_or(0);

                transaction.execute("INSERT INTO ShapeGroups (ShapeId, ParentShapeId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, parent_shape_idx, next_order]).map_err(|_| ())?;
            }
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Adds brush associations to a shape
    ///
    pub fn add_shape_brushes(&mut self, shape_id: CanvasShapeId, brush_ids: Vec<CanvasBrushId>) -> Result<(), ()> {
        let shape_idx               = self.index_for_shape(shape_id)?;
        let brush_indices: Vec<i64> = brush_ids.iter()
            .map(|brush_id| self.index_for_brush(*brush_id))
            .collect::<Result<Vec<_>, _>>()?;

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        let mut next_order: i64 = transaction.query_one("SELECT COALESCE(MAX(OrderIdx), -1) + 1 FROM ShapeBrushes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).map_err(|_| ())?;

        for brush_idx in brush_indices {
            transaction.execute("INSERT INTO ShapeBrushes (ShapeId, BrushId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, brush_idx, next_order]).map_err(|_| ())?;
            next_order += 1;
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Removes brush associations from a shape
    ///
    pub fn remove_shape_brushes(&mut self, shape_id: CanvasShapeId, brush_ids: Vec<CanvasBrushId>) -> Result<(), ()> {
        let shape_idx               = self.index_for_shape(shape_id)?;
        let brush_indices: Vec<i64> = brush_ids.iter()
            .map(|brush_id| self.index_for_brush(*brush_id))
            .collect::<Result<Vec<_>, _>>()?;

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        for brush_idx in brush_indices {
            if let Ok(order_idx) = transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeBrushes WHERE ShapeId = ? AND BrushId = ?", params![shape_idx, brush_idx], |row| row.get(0),
            ) {
                transaction.execute("DELETE FROM ShapeBrushes WHERE ShapeId = ? AND BrushId = ?", params![shape_idx, brush_idx]).map_err(|_| ())?;
                transaction.execute("UPDATE ShapeBrushes SET OrderIdx = OrderIdx - 1 WHERE ShapeId = ? AND OrderIdx > ?", params![shape_idx, order_idx]).map_err(|_| ())?;
            }
        }

        transaction.commit().map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Adds a brush to the canvas
    ///
    pub fn add_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), ()> {
        self.sqlite.execute("INSERT INTO Brushes (BrushGuid) VALUES (?)", params![brush_id.to_string()]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Removes a brush and all its associations from the canvas
    ///
    pub fn remove_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), ()> {
        self.sqlite.execute("DELETE FROM Brushes WHERE BrushGuid = ?", params![brush_id.to_string()]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Queries the outline of the document
    ///
    pub fn query_document_outline(&mut self, outline: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Add the document properties to start
        outline.push(VectorResponse::Document(vec![]));

        // Indicate the layers
        let mut layer_order     = vec![];

        // Layers are fetched in order
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY OrderIdx ASC").map_err(|_| ())?;
        let layers              = select_layers.query_map(params![], |row| Ok(row.get::<_, String>(0)?)).map_err(|_| ())?;

        for layer_row in layers {
            let layer_guid = layer_row.map_err(|_| ())?;
            let layer_guid = CanvasLayerId::from_string(&layer_guid);

            layer_order.push(layer_guid);
            outline.push(VectorResponse::Layer(layer_guid, vec![]));
        }

        outline.push(VectorResponse::LayerOrder(layer_order));
        Ok(())
    }
}
