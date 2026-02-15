use super::id_cache::*;
use super::super::brush::*;
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

    /// Reverse cache of the known property IDs
    pub (super) property_for_id_cache: HashMap<i64, CanvasPropertyId>,

    /// Cache of the known shape type IDs
    pub (super) shapetype_id_cache: HashMap<ShapeType, i64>,

    /// Reverse cache of the known shape type IDs
    pub (super) shapetype_for_id_cache: HashMap<i64, ShapeType>,

    /// Cache of the known shape IDs (maps to the index for the shape)
    pub (super) shape_id_cache: IdCache<CanvasShapeId, i64>,

    /// Cache of the known layer IDs (maps to the index for the layer)
    pub (super) layer_id_cache: HashMap<CanvasLayerId, i64>,

    /// The next shape ID to use (None if we haven't retrieved this from the database yet)
    next_shape_id: Option<i64>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Result<Self, ()> {
        sqlite.execute_batch("PRAGMA foreign_keys = ON").map_err(|_| ())?;

        Ok(Self {
            sqlite:                 sqlite,
            property_id_cache:      HashMap::new(),
            property_for_id_cache:  HashMap::new(),
            shapetype_id_cache:     HashMap::new(),
            shapetype_for_id_cache: HashMap::new(),
            shape_id_cache:         IdCache::new(200),
            layer_id_cache:         HashMap::new(),
            next_shape_id:          None,
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
                self.property_for_id_cache.insert(property_id, canvas_property_id);

                Ok(property_id)
            } else {
                // Create a new property ID
                let new_property_id = self.sqlite.query_one("INSERT INTO Properties (Name) VALUES (?) RETURNING PropertyId", [canvas_property_id.name()], |row| row.get::<_, i64>(0)).map_err(|_| ())?;
                self.property_id_cache.insert(canvas_property_id, new_property_id);
                self.property_for_id_cache.insert(new_property_id, canvas_property_id);

                Ok(new_property_id)
            }
        }
    }

    ///
    /// Retrieve or create a shape type ID in the database
    ///
    pub (super) fn index_for_shapetype(&mut self, shape_type: ShapeType) -> Result<i64, ()> {
        if let Some(cached_id) = self.shapetype_id_cache.get(&shape_type) {
            // We've encountered this shape type before so we know its ID
            Ok(*cached_id)
        } else {
            // Try to fetch the existing shape type
            let mut query_shapetype = self.sqlite.prepare_cached("SELECT ShapeTypeId FROM ShapeTypes WHERE Name = ?").map_err(|_| ())?;
            if let Ok(shapetype_id) = query_shapetype.query_one([shape_type.name()], |row| row.get::<_, i64>(0)) {
                // Cache it so we don't need to look it up again
                self.shapetype_id_cache.insert(shape_type, shapetype_id);
                self.shapetype_for_id_cache.insert(shapetype_id, shape_type);

                Ok(shapetype_id)
            } else {
                // Create a new shape type ID
                let new_shapetype_id = self.sqlite.query_one("INSERT INTO ShapeTypes (Name) VALUES (?) RETURNING ShapeTypeId", [shape_type.name()], |row| row.get::<_, i64>(0)).map_err(|_| ())?;
                self.shapetype_id_cache.insert(shape_type, new_shapetype_id);
                self.shapetype_for_id_cache.insert(new_shapetype_id, shape_type);

                Ok(new_shapetype_id)
            }
        }
    }

    ///
    /// Retrieve the property ID for a database index
    ///
    pub (super) fn property_for_index(&mut self, property_index: i64) -> Result<CanvasPropertyId, ()> {
        if let Some(cached_property) = self.property_for_id_cache.get(&property_index) {
            // We've encountered this index before so we know its property
            Ok(*cached_property)
        } else {
            // Fetch the property name from the database
            let mut query_name  = self.sqlite.prepare_cached("SELECT Name FROM Properties WHERE PropertyId = ?").map_err(|_| ())?;
            let name            = query_name.query_one([property_index], |row| row.get::<_, String>(0)).map_err(|_| ())?;

            // Create the property ID and cache it
            let canvas_property_id = CanvasPropertyId::new(&name);
            self.property_id_cache.insert(canvas_property_id, property_index);
            self.property_for_id_cache.insert(property_index, canvas_property_id);

            Ok(canvas_property_id)
        }
    }

    ///
    /// Retrieve the shape type for a database index
    ///
    pub (super) fn shapetype_for_index(&mut self, shapetype_index: i64) -> Result<ShapeType, ()> {
        if let Some(cached_shapetype) = self.shapetype_for_id_cache.get(&shapetype_index) {
            // We've encountered this index before so we know its shape type
            Ok(*cached_shapetype)
        } else {
            // Fetch the shape type name from the database
            let mut query_name  = self.sqlite.prepare_cached("SELECT Name FROM ShapeTypes WHERE ShapeTypeId = ?").map_err(|_| ())?;
            let name            = query_name.query_one([shapetype_index], |row| row.get::<_, String>(0)).map_err(|_| ())?;

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
    fn shapetype_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<ShapeType, ()> {
        let mut shape_type_query    = self.sqlite.prepare_cached("SELECT ShapeType FROM Shapes WHERE ShapeGuid = ?").map_err(|_| ())?;
        let shape_type              = shape_type_query.query_one(params![shape_id.to_string()], |row| row.get(0)).map_err(|_| ())?;
        drop(shape_type_query);

        self.shapetype_for_index(shape_type)
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_layer(&mut self, layer_id: CanvasLayerId) -> Result<i64, ()> {
        if let Some(cached_id) = self.layer_id_cache.get(&layer_id) {
            Ok(*cached_id)
        } else {
            let idx = self.sqlite.query_one::<i64, _, _>("SELECT LayerId FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).map_err(|_| ())?;
            self.layer_id_cache.insert(layer_id, idx);
            Ok(idx)
        }
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<i64, ()> {
        if let Some(cached_id) = self.shape_id_cache.get(&shape_id) {
            Ok(*cached_id)
        } else {
            let idx = self.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [shape_id.to_string()], |row| row.get(0)).map_err(|_| ())?;
            self.shape_id_cache.insert(shape_id, idx);
            Ok(idx)
        }
    }

    ///
    /// Retrieves the time (in nanoseconds) when a shape appears on the canvas
    ///
    #[inline]
    pub fn time_for_shape(&mut self, shape_id: CanvasShapeId) -> Result<i64, ()> {
        let mut time_query = self.sqlite.prepare("
            SELECT      sl.Time 
            FROM        ShapeLayers sl 
            INNER JOIN  Shapes s ON s.ShapeId = sl.ShapeId 
            WHERE       s.ShapeGuid = ?").map_err(|_| ())?;

        let mut time    = time_query.query_map([shape_id.to_string()], |row| row.get(0)).map_err(|_| ())?;
        let time        = time.next().unwrap_or(Ok(0i64)).map_err(|_| ())?;

        Ok(time)
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
    /// Collects all descendents of a shape in depth-first pre-order (does not include the shape itself).
    ///
    #[inline]
    fn all_descendents_for_shape(transaction: &Transaction<'_>, shape_idx: i64) -> Result<Vec<i64>, ()> {
        let mut result = Vec::new();
        Self::collect_shape_dependents(transaction, shape_idx, &mut result)?;
        Ok(result)
    }

    ///
    /// Recurses through the descendents of a shape
    ///
    fn collect_shape_dependents(transaction: &Transaction<'_>, parent_idx: i64, result: &mut Vec<i64>) -> Result<(), ()> {
        // Using a recursive Rust function because SQL CTEs don't guarantee depth-first ordering.
        let mut stmt = transaction.prepare_cached("SELECT ShapeId FROM ShapeGroups WHERE ParentShapeId = ? ORDER BY OrderIdx ASC").map_err(|_| ())?;
        let children: Vec<i64> = stmt.query_map(params![parent_idx], |row| row.get(0))
            .map_err(|_| ())?
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
    /// Inserts a block of shapes into ShapeLayers at the specified position, shifting existing entries to make room.
    ///
    fn insert_shapes_on_layer(transaction: &Transaction<'_>, layer_id: i64, at_order: i64, shape_ids: &[i64], time: i64) -> Result<(), ()> {
        let block_size = shape_ids.len() as i64;

        // Make room for the block
        transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx + ? WHERE LayerId = ? AND OrderIdx >= ?", params![block_size, layer_id, at_order]).map_err(|_| ())?;

        // Insert each shape
        let mut insert = transaction.prepare_cached("INSERT INTO ShapeLayers (ShapeId, LayerId, OrderIdx, Time) VALUES (?, ?, ?, ?)").map_err(|_| ())?;
        for (i, shape_id) in shape_ids.iter().enumerate() {
            insert.execute(params![shape_id, layer_id, at_order + i as i64, time]).map_err(|_| ())?;
        }

        Ok(())
    }

    ///
    /// Removes a contiguous block of entries from ShapeLayers and compacts the ordering.
    ///
    fn remove_shapes_from_layer(transaction: &Transaction<'_>, layer_id: i64, from_order: i64, block_size: i64) -> Result<(), ()> {
        // Delete the block
        transaction.execute("DELETE FROM ShapeLayers WHERE LayerId = ? AND OrderIdx >= ? AND OrderIdx < ?", params![layer_id, from_order, from_order + block_size]).map_err(|_| ())?;

        // Compact the ordering
        transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - ? WHERE LayerId = ? AND OrderIdx >= ?", params![block_size, layer_id, from_order + block_size]).map_err(|_| ())?;

        Ok(())
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
    /// Deletes properties of the specified type from the canvas
    ///
    pub fn delete_properties(&mut self, target: CanvasPropertyTarget, properties: Vec<CanvasPropertyId>) -> Result<(), ()> {
        // Map to property IDs
        let properties = properties.into_iter()
            .map(|property_id| self.index_for_property(property_id))
            .collect::<Result<Vec<_>, _>>()?;

        // Deletion operation depends on the target
        match target {
            CanvasPropertyTarget::Document          => { self.delete_sql_properties("DELETE FROM Document{}Properties WHERE PropertyId = ?", properties.into_iter(), vec![])?; },
            CanvasPropertyTarget::Layer(layer_id)   => { let layer_idx = self.index_for_layer(layer_id)?; self.delete_sql_properties("DELETE FROM Layer{}Properties WHERE LayerId = ? AND PropertyId = ?", properties.into_iter(), vec![&layer_idx])?; },
            CanvasPropertyTarget::Brush(brush_id)   => { let brush_idx = self.index_for_brush(brush_id)?; self.delete_sql_properties("DELETE FROM Brush{}Properties WHERE BrushId = ? AND PropertyId = ?", properties.into_iter(), vec![&brush_idx])?; },
            CanvasPropertyTarget::Shape(shape_id)   => { let shape_idx = self.index_for_shape(shape_id)?; self.delete_sql_properties("DELETE FROM Shape{}Properties WHERE ShapeId = ? AND PropertyId = ?", properties.into_iter(), vec![&shape_idx])?; },
        }

        Ok(())
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
    /// Removes the a set of property IDs from one of the property tables, using a template
    ///
    /// The template should be something like `DELETE FROM Shape{}Properties WHERE ShapeId = ? AND PropertyId = ?`. The property ID is appended to other_params
    /// for each table
    ///
    #[inline]
    fn delete_sql_properties(&mut self, command_template: &str, properties: impl Iterator<Item=i64>, other_params: Vec<&dyn ToSql>) -> Result<(), ()> {
        // Delete all or nothing
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Prepare the three commands to delete the three different types of property
        let delete_ints     = command_template.replace("{}", "Int");
        let delete_floats   = command_template.replace("{}", "Float");
        let delete_blobs    = command_template.replace("{}", "Blob");

        let mut delete_ints     = transaction.prepare_cached(&delete_ints).map_err(|_| ())?;
        let mut delete_floats   = transaction.prepare_cached(&delete_floats).map_err(|_| ())?;
        let mut delete_blobs    = transaction.prepare_cached(&delete_blobs).map_err(|_| ())?;

        // Delete the properties in the list
        for property_idx in properties {
            // Append the property index to the params (so it's the last parameter)
            let mut params = other_params.clone();
            params.extend(params![property_idx]);
            let params: &[&dyn ToSql] = &params;

            // Delete from the three properties tables
            delete_ints.execute(params).map_err(|_| ())?;
            delete_floats.execute(params).map_err(|_| ())?;
            delete_blobs.execute(params).map_err(|_| ())?;
        }

        // Finish up the transaction
        drop(delete_ints);
        drop(delete_floats);
        drop(delete_blobs);

        transaction.commit().map_err(|_| ())?;

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
        let new_layer_idx: i64 = transaction.query_one("INSERT INTO Layers(LayerGuid, OrderIdx) VALUES (?, ?) RETURNING LayerId", params![new_layer_id.to_string(), new_layer_order], |row| row.get(0)).map_err(|_| ())?;

        transaction.commit().map_err(|_| ())?;

        self.layer_id_cache.insert(new_layer_id, new_layer_idx);

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

        self.layer_id_cache.remove(&old_layer_id);

        Ok(())
    }

    ///
    /// Adds a frame to a layer at the specified time with the specified length
    ///
    pub fn add_frame(&mut self, frame_layer: CanvasLayerId, when: Duration, _length: Duration) -> Result<(), ()> {
        let layer_idx   = self.index_for_layer(frame_layer)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("INSERT INTO LayerFrames (LayerId, Time) VALUES (?, ?)", params![layer_idx, when_nanos]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Removes a frame from a layer at the specified time
    ///
    pub fn remove_frame(&mut self, frame_layer: CanvasLayerId, when: Duration) -> Result<(), ()> {
        // TODO: also remove the shapes that exist in this timeframe

        let layer_idx   = self.index_for_layer(frame_layer)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("DELETE FROM LayerFrames WHERE LayerId = ? AND Time = ?", params![layer_idx, when_nanos]).map_err(|_| ())?;

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
    pub fn add_shape(&mut self, shape_id: CanvasShapeId, shape_type: ShapeType, shape: CanvasShape) -> Result<(), ()> {
        let shape_type_idx                  = self.index_for_shapetype(shape_type)?;
        let (shape_data_type, shape_data)   = Self::encode_shape(&shape)?;

        if let Ok(existing_idx) = self.index_for_shape(shape_id) {
            // Replace the existing shape definition in place
            let mut update_existing = self.sqlite.prepare_cached("UPDATE Shapes SET ShapeType = ?, ShapeDataType = ?, ShapeData = ? WHERE ShapeId = ?").map_err(|_| ())?;
            update_existing.execute(params![shape_type_idx, shape_data_type, shape_data, existing_idx]).map_err(|_| ())?;
        } else {
            // Insert a new shape with a generated ShapeId
            let mut insert_new  = self.sqlite.prepare_cached("INSERT INTO Shapes (ShapeId, ShapeGuid, ShapeType, ShapeDataType, ShapeData) VALUES (?, ?, ?, ?, ?)").map_err(|_| ())?;
            let next_id: i64    = if let Some(cached_id) = self.next_shape_id {
                cached_id
            } else {
                let mut get_max_id = self.sqlite.prepare_cached("SELECT COALESCE(MAX(ShapeId), 0) + 1 FROM Shapes").map_err(|_| ())?;
                get_max_id.query_one([], |row| row.get(0)).map_err(|_| ())?
            };

            insert_new.execute(params![next_id, shape_id.to_string(), shape_type_idx, shape_data_type, shape_data]).map_err(|_| ())?;

            // Store the shape ID in the cache so we can look it up faster for things like setting properties
            self.shape_id_cache.insert(shape_id, next_id);
            self.next_shape_id = Some(next_id + 1);
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

        // Collect descendents for block-size compaction and deletion
        let descendents = Self::all_descendents_for_shape(&transaction, shape_idx)?;
        let block_size  = 1 + descendents.len() as i64;

        // Delete all descendents from Shapes (CASCADE handles their ShapeLayers, ShapeGroups, properties)
        for desc_id in &descendents {
            transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![desc_id]).map_err(|_| ())?;
        }

        // Delete the shape: CASCADE handles ShapeLayers, ShapeGroups, ShapeBrushes, and properties
        transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;

        // Compact ordering in the parent layer by block_size (shape + all descendents were contiguous)
        if let Some((layer_id, order_idx)) = layer_info {
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - ? WHERE LayerId = ? AND OrderIdx > ?", params![block_size, layer_id, order_idx]).map_err(|_| ())?;
        }

        // Compact ordering in the parent group by 1 (only the shape itself was a direct child)
        if let Some((parent_id, order_idx)) = group_info {
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx]).map_err(|_| ())?;
        }

        transaction.commit().map_err(|_| ())?;

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
    pub fn set_shape_definition(&mut self, shape_id: CanvasShapeId, shape: CanvasShape) -> Result<(), ()> {
        let shape_idx                   = self.index_for_shape(shape_id)?;
        let (shape_type, shape_data)    = Self::encode_shape(&shape)?;

        self.sqlite.execute("UPDATE Shapes SET ShapeDataType = ?, ShapeData = ? WHERE ShapeId = ?", params![shape_type, shape_data, shape_idx]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Sets the time when a shape should appear on its layer
    ///
    pub fn set_shape_time(&mut self, shape_id: CanvasShapeId, when: Duration) -> Result<(), ()> {
        let shape_idx   = self.index_for_shape(shape_id)?;
        let when_nanos  = when.as_nanos() as i64;

        self.sqlite.execute("UPDATE ShapeLayers SET Time = ? WHERE ShapeId = ?", params![when_nanos, shape_idx]).map_err(|_| ())?;

        Ok(())
    }

    ///
    /// Reorders a shape within its current parent (layer or group)
    ///
    pub fn reorder_shape(&mut self, shape_id: CanvasShapeId, before_shape: Option<CanvasShapeId>) -> Result<(), ()> {
        let shape_idx           = self.index_for_shape(shape_id)?;
        let before_shape_idx    = before_shape.map(|bs| self.index_for_shape(bs)).transpose()?;

        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Check if shape is in a group first
        if let Ok((parent_id, original_order)) = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            // Read the existing order of the shape within a group
            let before_order = if let Some(before_idx) = before_shape_idx {
                transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeGroups WHERE ShapeId = ? AND ParentShapeId = ?", params![before_idx, parent_id], |row| row.get(0)).map_err(|_| ())?
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_id], |row| row.get(0)).map_err(|_| ())?;
                max_order.map(|idx| idx + 1).unwrap_or(0)
            };

            // Reorder within ShapeGroups
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, original_order]).map_err(|_| ())?;
            let before_order = if before_order > original_order { before_order - 1 } else { before_order };

            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx + 1 WHERE ParentShapeId = ? AND OrderIdx >= ?", params![parent_id, before_order]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = ? WHERE ShapeId = ?", params![before_order, shape_idx]).map_err(|_| ())?;

            // Rebuild the parent group's ShapeLayers descendents to reflect the new ordering
            if let Ok((layer_id, parent_order_idx, parent_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![parent_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))) {
                // Collect new depth-first order (ShapeGroups already reordered)
                let new_descendents = Self::all_descendents_for_shape(&transaction, parent_id)?;
                let desc_count      = new_descendents.len() as i64;

                // Remove old descendent entries from ShapeLayers
                Self::remove_shapes_from_layer(&transaction, layer_id, parent_order_idx + 1, desc_count)?;

                // Re-insert in new depth-first order, preserving the parent group's time
                Self::insert_shapes_on_layer(&transaction, layer_id, parent_order_idx + 1, &new_descendents, parent_time)?;
            }

            transaction.commit().map_err(|_| ())?;
            return Ok(());
        }

        // Check if shape is directly on a layer (not in a group)
        if let Ok((layer_id, original_order, original_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))) {
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
                let (order, time) = transaction.query_one("SELECT OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ? AND LayerId = ?", params![before_idx, layer_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).map_err(|_| ())?;
                (order, time)
            } else {
                let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_id], |row| row.get(0)).map_err(|_| ())?;
                (max_order.map(|idx| idx + 1).unwrap_or(0), original_time)
            };

            // Re-insert the block at the new position with the target time
            Self::insert_shapes_on_layer(&transaction, layer_id, before_order, &block, new_time)?;

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
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        // Collect descendents for block operations (the shape and its descendents move together)
        let descendents = Self::all_descendents_for_shape(&transaction, shape_idx)?;
        let block_size  = 1 + descendents.len() as i64;

        // Remove from ShapeLayers (covers both direct layer parent and group-via-layer)
        if let Ok((layer_id, order_idx)) = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            Self::remove_shapes_from_layer(&transaction, layer_id, order_idx, block_size)?;
        }

        // Remove from any existing group parent
        if let Ok((parent_id, order_idx)) = transaction.query_one(
            "SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?",
            params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ) {
            transaction.execute("DELETE FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx]).map_err(|_| ())?;
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
                let next_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeLayers WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).map_err(|_| ())?;
                let next_order = next_order.map(|idx| idx + 1).unwrap_or(0);

                Self::insert_shapes_on_layer(&transaction, layer_idx, next_order, &block, when_nanos)?;
            }

            CanvasShapeParent::Shape(_) => {
                let parent_shape_idx = new_parent_shape_idx.unwrap();

                // Count the parent group's current descendents before inserting (for finding the insertion point in ShapeLayers)
                let parent_old_descendents = Self::all_descendents_for_shape(&transaction, parent_shape_idx)?;
                let parent_old_block_size  = 1 + parent_old_descendents.len() as i64;

                // Add to ShapeGroups at the end
                let next_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM ShapeGroups WHERE ParentShapeId = ?", params![parent_shape_idx], |row| row.get(0)).map_err(|_| ())?;
                let next_order = next_order.map(|idx| idx + 1).unwrap_or(0);

                transaction.execute("INSERT INTO ShapeGroups (ShapeId, ParentShapeId, OrderIdx) VALUES (?, ?, ?)", params![shape_idx, parent_shape_idx, next_order]).map_err(|_| ())?;

                // Also add to ShapeLayers if the parent group is on a layer
                if let Ok((layer_id, parent_sl_order, parent_time)) = transaction.query_one("SELECT LayerId, OrderIdx, Time FROM ShapeLayers WHERE ShapeId = ?", params![parent_shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))) {
                    let insert_at = parent_sl_order + parent_old_block_size;
                    Self::insert_shapes_on_layer(&transaction, layer_id, insert_at, &block, parent_time)?;
                }
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
            if let Ok(order_idx) = transaction.query_one::<i64, _, _>("SELECT OrderIdx FROM ShapeBrushes WHERE ShapeId = ? AND BrushId = ?", params![shape_idx, brush_idx], |row| row.get(0)) {
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
    /// Returns the shape IDs that have the specified brush attached
    ///
    pub fn shapes_with_brush(&self, brush_id: CanvasBrushId) -> Result<Vec<CanvasShapeId>, ()> {
        let mut query_shapes = self.sqlite.prepare_cached("SELECT s.ShapeGuid FROM ShapeBrushes sb JOIN Brushes b ON sb.BrushId = b.BrushId JOIN Shapes s ON sb.ShapeId = s.ShapeId WHERE b.BrushGuid = ?").map_err(|_| ())?;

        let mut shapes = vec![];
        for row in query_shapes.query_map(params![brush_id.to_string()], |row| Ok(CanvasShapeId::from_string(&row.get::<_, String>(0)?))).map_err(|_| ())? {
            shapes.push(row.map_err(|_| ())?);
        }

        Ok(shapes)
    }

    ///
    /// Given a row returned from the database with the int, float and blob values for a property at the start, in
    /// that order, decodes the canvas property corresponding to that row.
    ///
    fn decode_property(row: &Row) -> Option<CanvasProperty> {
        if let Some(int_val) = row.get::<_, Option<i64>>(0).ok()? {
            Some(CanvasProperty::Int(int_val))
        } else if let Some(float_val) = row.get::<_, Option<f64>>(1).ok()? {
            Some(CanvasProperty::Float(float_val as _))
        } else if let Some(blob_val) = row.get::<_, Option<Vec<u8>>>(2).ok()? {
            Some(postcard::from_bytes(&blob_val).ok()?)
        } else {
            None
        }
    }

    ///
    /// Reads the document properties generate s a VectorResponse::Document in the response
    ///
    pub fn query_document(&mut self, document_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Read the properties from the three properties tables
        let mut q_int_properties    = self.sqlite.prepare_cached("SELECT IntValue,   PropertyId FROM DocumentIntProperties").map_err(|_| ())?;
        let mut q_float_properties  = self.sqlite.prepare_cached("SELECT FloatValue, PropertyId FROM DocumentFloatProperties").map_err(|_| ())?;
        let mut q_blob_properties   = self.sqlite.prepare_cached("SELECT BlobValue,  PropertyId FROM DocumentBlobProperties").map_err(|_| ())?;

        let int_properties          = q_int_properties.query_map([],   |row| Ok((row.get::<_, i64>(1)?, CanvasProperty::Int(row.get(0)?)))).map_err(|_| ())?.collect::<Result<Vec<_>, _>>().map_err(|_| ())?;
        let float_properties        = q_float_properties.query_map([], |row| Ok((row.get::<_, i64>(1)?, CanvasProperty::Float(row.get(0)?)))).map_err(|_| ())?.collect::<Result<Vec<_>, _>>().map_err(|_| ())?;
        let blob_properties         = q_blob_properties.query_map([],  |row| Ok((row.get::<_, i64>(1)?, postcard::from_bytes::<CanvasProperty>(&row.get::<_, Vec<u8>>(0)?).ok()))).map_err(|_| ())?.collect::<Result<Vec<_>, _>>().map_err(|_| ())?;

        drop(q_blob_properties);
        drop(q_float_properties);
        drop(q_int_properties);

        // Combine them to create the final set of document properties
        let document_properties = int_properties.into_iter()
            .chain(float_properties.into_iter())
            .chain(blob_properties.into_iter().flat_map(|(property_idx, maybe_value)| Some((property_idx, maybe_value?))))
            .flat_map(|(property_idx, value)| Some((self.property_for_index(property_idx).ok()?, value)))
            .collect::<Vec<_>>();

        // Add the response
        document_response.push(VectorResponse::Document(document_properties));

        Ok(())
    }

    ///
    /// Queries a list of layers for their properties
    ///
    pub fn query_layers(&mut self, query_layers: impl IntoIterator<Item=CanvasLayerId>, layer_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Query to fetch the properties ofr each layer
        let properties_query =
            "
            SELECT ip.IntValue, fp.FloatValue, bp.BlobValue, COALESCE(ip.PropertyId, fp.PropertyId, bp.PropertyId)
            FROM Layers l
            LEFT OUTER JOIN LayerIntProperties   ip ON ip.LayerId = l.LayerId
            LEFT OUTER JOIN LayerFloatProperties fp ON fp.LayerId = l.LayerId
            LEFT OUTER JOIN LayerBlobProperties  bp ON bp.LayerId = l.LayerId
            WHERE l.LayerGuid = ?
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query).map_err(|_| ())?;

        // We query layers one at a time (would be nice if we could pass in an array to Sqlite)
        let mut layers = vec![];

        // Read the property values
        for layer in query_layers {
            // Read the properties for this layer
            let properties = properties_query.query_map(params![layer.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                }).map_err(|_| ())?
                .flatten()
                .flat_map(|(property_idx, value)| Some((property_idx, value?)))
                .collect::<Vec<_>>();

            layers.push((layer, properties));
        }

        drop(properties_query);

        // Map the property indexes to the actual property values, then generate the layer response
        for (layer, properties) in layers.into_iter() {
            let properties = properties.into_iter()
                .map(|(property_idx, value)| Ok((self.property_for_index(property_idx)?, value)))
                .collect::<Result<Vec<_>, _>>()?;

            layer_response.push(VectorResponse::Layer(layer, properties));
        }

        Ok(())
    }

    ///
    /// Queries a list of brushes for their properties
    ///
    pub fn query_brushes(&mut self, query_brushes: impl IntoIterator<Item=CanvasBrushId>, brush_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Query to fetch the properties ofr each brush
        let properties_query =
            "
            SELECT ip.IntValue, fp.FloatValue, bp.BlobValue, COALESCE(ip.PropertyId, fp.PropertyId, bp.PropertyId)
            FROM Brushes b
            LEFT OUTER JOIN BrushIntProperties   ip ON ip.BrushId = b.BrushId
            LEFT OUTER JOIN BrushFloatProperties fp ON fp.BrushId = b.BrushId
            LEFT OUTER JOIN BrushBlobProperties  bp ON bp.BrushId = b.BrushId
            WHERE b.BrushGuid = ?
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query).map_err(|_| ())?;

        // We query brushes one at a time (would be nice if we could pass in an array to Sqlite)
        let mut brushes = vec![];

        // Read the property values
        for brush_id in query_brushes {
            // Read the properties for this brush
            let properties = properties_query.query_map(params![brush_id.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                }).map_err(|_| ())?
                .flatten()
                .flat_map(|(property_idx, value)| Some((property_idx, value?)))
                .collect::<Vec<_>>();

            brushes.push((brush_id, properties));
        }

        drop(properties_query);

        // Map the property indexes to the actual property values, then generate the layer response
        for (brush_id, properties) in brushes.into_iter() {
            let properties = properties.into_iter()
                .map(|(property_idx, value)| Ok((self.property_for_index(property_idx)?, value)))
                .collect::<Result<Vec<_>, _>>()?;

            brush_response.push(VectorResponse::Brush(brush_id, properties));
        }

        Ok(())
    }

    ///
    /// Queries a list of shapes for their properties
    ///
    pub fn query_shapes(&mut self, query_shapes: impl IntoIterator<Item=CanvasShapeId>, shape_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Query to fetch the properties for each shape, including brush properties from attached brushes
        let properties_query =
            "
            SELECT ip.IntValue, fp.FloatValue, bp.BlobValue, COALESCE(ip.PropertyId, fp.PropertyId, bp.PropertyId) AS PropertyId, 0 AS Source, 0 AS BrushOrder
            FROM Shapes s
            LEFT OUTER JOIN ShapeIntProperties   ip ON ip.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeFloatProperties fp ON fp.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeBlobProperties  bp ON bp.ShapeId = s.ShapeId
            WHERE s.ShapeGuid = ?1

            UNION ALL

            SELECT bip.IntValue, bfp.FloatValue, bbp.BlobValue, COALESCE(bip.PropertyId, bfp.PropertyId, bbp.PropertyId) AS PropertyId, 1 AS Source, sb.OrderIdx AS BrushOrder
            FROM Shapes s
            INNER JOIN      ShapeBrushes         sb ON sb.ShapeId = s.ShapeId
            LEFT OUTER JOIN BrushIntProperties   bip ON bip.BrushId = sb.BrushId
            LEFT OUTER JOIN BrushFloatProperties bfp ON bfp.BrushId = sb.BrushId
            LEFT OUTER JOIN BrushBlobProperties  bbp ON bbp.BrushId = sb.BrushId
            WHERE s.ShapeGuid = ?1

            ORDER BY PropertyId ASC, Source ASC, BrushOrder DESC
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query).map_err(|_| ())?;

        let mut shapes = vec![];

        // Read the property values
        for shape in query_shapes {
            // Read the properties for this shape
            let properties = properties_query.query_map(params![shape.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                }).map_err(|_| ())?
                .flatten()
                .flat_map(|(property_idx, value)| Some((property_idx, value?)))
                .fold(vec![], |mut properties: Vec<(i64, CanvasProperty)>, (property_idx, value)| {
                    // Only keep the first value for each property index (results are ordered so the shape property comes before any brush properties)
                    if properties.last().map_or(true, |(last_idx, _)| *last_idx != property_idx) {
                        properties.push((property_idx, value));
                    }
                    properties
                });

            shapes.push((shape, properties));
        }

        drop(properties_query);

        // Map the property indexes to the actual property values, then generate the shape responses
        for (shape_id, properties) in shapes.into_iter() {
            let properties = properties.into_iter()
                .map(|(property_idx, value)| Ok((self.property_for_index(property_idx)?, value)))
                .collect::<Result<Vec<_>, _>>()?;
            let shape_type = self.shapetype_for_shape(shape_id)?;

            shape_response.push(VectorResponse::Shape(shape_id, shape_type, properties));
        }

        Ok(())
    }

    ///
    /// Queries the shapes and their properties on a particular layer
    ///
    pub fn query_shapes_on_layer(&mut self, layer: CanvasLayerId, shape_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Query to fetch the properties for each shape, including brush properties from attached brushes
        let shapes_query =
            "
            SELECT 
                ip.IntValue, fp.FloatValue, bp.BlobValue, COALESCE(ip.PropertyId, fp.PropertyId, bp.PropertyId) AS PropertyId, 
                0 AS Source, 0 AS BrushOrder, sl.OrderIdx As ShapeOrder, s.ShapeId As ShapeIdx, s.ShapeGuid As ShapeGuid, 
                g.ParentShapeId As GroupIdx, s.ShapeType AS ShapeType
            FROM Shapes s
            INNER JOIN      ShapeLayers          sl ON sl.ShapeId = s.ShapeId
            INNER JOIN      Layers               l  ON l.LayerId = sl.LayerId
            LEFT OUTER JOIN ShapeGroups          g  ON g.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeIntProperties   ip ON ip.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeFloatProperties fp ON fp.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeBlobProperties  bp ON bp.ShapeId = s.ShapeId
            WHERE l.LayerGuid = ?1

            UNION ALL

            SELECT 
                bip.IntValue, bfp.FloatValue, bbp.BlobValue, COALESCE(bip.PropertyId, bfp.PropertyId, bbp.PropertyId) AS PropertyId,
                1 AS Source, sb.OrderIdx AS BrushOrder, sl.OrderIdx As ShapeOrder, s.ShapeId As ShapeIdx, s.ShapeGuid As ShapeGuid, 
                g.ParentShapeId As GroupIdx, s.ShapeType AS ShapeType
            FROM Shapes s
            INNER JOIN      ShapeLayers          sl ON sl.ShapeId = s.ShapeId
            INNER JOIN      Layers               l  ON l.LayerId = sl.LayerId
            INNER JOIN      ShapeBrushes         sb ON sb.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeGroups          g  ON g.ShapeId = s.ShapeId
            LEFT OUTER JOIN BrushIntProperties   bip ON bip.BrushId = sb.BrushId
            LEFT OUTER JOIN BrushFloatProperties bfp ON bfp.BrushId = sb.BrushId
            LEFT OUTER JOIN BrushBlobProperties  bbp ON bbp.BrushId = sb.BrushId
            WHERE l.LayerGuid = ?1

            ORDER BY ShapeOrder ASC, PropertyId ASC, Source ASC, BrushOrder DESC
            ";
        let mut shapes_query = self.sqlite.prepare_cached(shapes_query).map_err(|_| ())?;

        // Shapes we've seen from the query, and the properties we've gathered from each shape
        let mut shapes      = vec![];
        let mut properties  = vec![];

        let mut shapes_rows      = shapes_query.query(params![layer.to_string()]).map_err(|_| ())?;
        let mut cur_shape_idx    = None;
        let mut cur_shape_id     = None;
        let mut cur_shape_type   = None;
        let mut cur_property_idx = None;
        let mut cur_group        = None;

        while let Ok(Some(shape_row)) = shapes_rows.next() {
            // Update the shape that we're reading
            let shape_idx = shape_row.get::<_, i64>(7).map_err(|_| ())?;
            let group_idx = shape_row.get::<_, Option<i64>>(9).map_err(|_| ())?;
            if Some(shape_idx) != cur_shape_idx {
                // Finished receiving properties for the current shape, so move on to the next
                if let (Some(old_shape_id), Some(old_shape_idx), Some(old_shape_type)) = (cur_shape_id, cur_shape_idx, cur_shape_type) {
                    // cur_group contains the old group at this point
                    shapes.push((old_shape_idx, old_shape_id, old_shape_type, properties, cur_group));
                    properties = vec![];
                }

                // Update to track the shape indicated by the current row
                cur_shape_idx    = Some(shape_idx);
                cur_shape_id     = Some(CanvasShapeId::from_string(&shape_row.get::<_, String>(8).map_err(|_| ())?));
                cur_shape_type   = Some(shape_row.get::<_, i64>(10).map_err(|_| ())?);
                cur_property_idx = None;
                cur_group        = group_idx;
            }

            // Read the next property
            if let Some(property_value) = Self::decode_property(&shape_row) {
                let property_idx = shape_row.get::<_, i64>(3).map_err(|_| ())?;

                if Some(property_idx) != cur_property_idx {
                    // Only write the first property if the property is defined in more than one place
                    cur_property_idx = Some(property_idx);
                    properties.push((property_idx, property_value));
                }
            }
        }

        // Set the last shape
        if let (Some(shape_id), Some(shape_idx), Some(shape_type)) = (cur_shape_id, cur_shape_idx, cur_shape_type) {
            shapes.push((shape_idx, shape_id, shape_type, properties, cur_group));
        }

        drop(shapes_rows);
        drop(shapes_query);

        // Generate the shapes for the response
        let mut last_group_idx  = None;
        let mut last_shape_idx  = None;
        let mut group_stack     = vec![];

        for (shape_idx, shape_id, shape_type_idx, properties, group_idx) in shapes.into_iter() {
            // Generate start or end group messages
            // We can either be entering a group for the previous shape (new group ID matching the last shape we generated), or 
            // leaving to a parent group (new group ID not matching the last shape we generated)
            // Groups always have a 'parent' shape so the case where a group changes to a shape we haven't already seen in this 
            // way should be impossible
            if group_idx != last_group_idx {
                // Group has changed
                if let Some(group_idx) = group_idx {
                    // Changed to either a new child group or a parent group
                    if Some(group_idx) == last_shape_idx {
                        // Moving further down the stack
                        group_stack.push(group_idx);
                        shape_response.push(VectorResponse::StartGroup);
                    } else {
                        // Leaving to a parent group
                        while group_stack.last() != Some(&group_idx) {
                            if group_stack.pop().is_none() {
                                // Oops: if we get here there's some corruption in the shape ordering
                                debug_assert!(false, "Moved to a group that's not a child or a parent of the current shape");
                                break;
                            }

                            shape_response.push(VectorResponse::EndGroup);
                        }
                    }
                } else {
                    // Left all groups and back at the main layer
                    while group_stack.pop().is_some() {
                        shape_response.push(VectorResponse::EndGroup);
                    }
                }
            }

            // Map the shape type and properties
            let shape_type = self.shapetype_for_index(shape_type_idx)?;
            let properties = properties.into_iter()
                .map(|(property_idx, value)| Ok((self.property_for_index(property_idx)?, value)))
                .collect::<Result<Vec<_>, _>>()?;

            shape_response.push(VectorResponse::Shape(shape_id, shape_type, properties));

            last_group_idx  = group_idx;
            last_shape_idx  = Some(shape_idx);
        }

        Ok(())
    }

    ///
    /// Queries a list of layers, retrieving their properties and any shapes that are on them
    ///
    pub fn query_layers_with_shapes(&mut self, query_layers: impl IntoIterator<Item=CanvasLayerId>, layer_response: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Query the layers
        let mut layers = vec![];
        self.query_layers(query_layers, &mut layers)?;

        // Merge in the shapes to generate the result
        for response in layers {
            match response {
                VectorResponse::Layer(layer_id, layer_properties) => {
                    layer_response.push(VectorResponse::Layer(layer_id, layer_properties));
                    self.query_shapes_on_layer(layer_id, layer_response)?;
                }

                other => {
                    layer_response.push(other);
                }
            }
        }

        Ok(())
    }

    ///
    /// Queries the outline of the document
    ///
    pub fn query_document_outline(&mut self, outline: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Add the document properties to start
        self.query_document(outline)?;

        // Layers are fetched in order
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY OrderIdx ASC").map_err(|_| ())?;
        let layers              = select_layers.query_map(params![], |row| Ok(CanvasLayerId::from_string(&row.get::<_, String>(0)?)))
            .map_err(|_| ())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ())?;
        drop(select_layers);

        // Use the layer query to populate the layers, then write the order
        self.query_layers(layers.iter().copied(), outline)?;
        outline.push(VectorResponse::LayerOrder(layers));

        Ok(())
    }

    ///
    /// Queries the whole of the document
    ///
    pub fn query_document_whole(&mut self, outline: &mut Vec<VectorResponse>) -> Result<(), ()> {
        // Add the document properties to start
        self.query_document(outline)?;

        // Layers are fetched in order
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY OrderIdx ASC").map_err(|_| ())?;
        let layers              = select_layers.query_map(params![], |row| Ok(CanvasLayerId::from_string(&row.get::<_, String>(0)?)))
            .map_err(|_| ())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ())?;
        drop(select_layers);

        // Use the layer query to populate the layers, then write the order
        self.query_layers_with_shapes(layers.iter().copied(), outline)?;
        outline.push(VectorResponse::LayerOrder(layers));

        // Query the brushes
        let mut select_brushes   = self.sqlite.prepare_cached("SELECT BrushGuid FROM Brushes").map_err(|_| ())?;
        let brushes              = select_brushes.query_map(params![], |row| Ok(CanvasBrushId::from_string(&row.get::<_, String>(0)?)))
            .map_err(|_| ())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ())?;
        drop(select_brushes);
        self.query_brushes(brushes, outline)?;

        Ok(())
    }
}
