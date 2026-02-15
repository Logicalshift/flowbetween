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
    /// Retrieve or create a property ID in the database
    ///
    pub (super) fn index_for_property(&mut self, canvas_property_id: CanvasPropertyId) -> Result<i64, CanvasError> {
        if let Some(cached_id) = self.property_id_cache.get(&canvas_property_id) {
            // We've encountered this property before so we know its ID
            Ok(*cached_id)
        } else {
            // Try to fetch the existing property
            let mut query_property = self.sqlite.prepare_cached("SELECT PropertyId FROM Properties WHERE Name = ?")?;
            if let Ok(property_id) = query_property.query_one([canvas_property_id.name()], |row| row.get::<_, i64>(0)) {
                // Cache it so we don't need to look it up again
                self.property_id_cache.insert(canvas_property_id, property_id);
                self.property_for_id_cache.insert(property_id, canvas_property_id);

                Ok(property_id)
            } else {
                // Create a new property ID
                let new_property_id = self.sqlite.query_one("INSERT INTO Properties (Name) VALUES (?) RETURNING PropertyId", [canvas_property_id.name()], |row| row.get::<_, i64>(0))?;
                self.property_id_cache.insert(canvas_property_id, new_property_id);
                self.property_for_id_cache.insert(new_property_id, canvas_property_id);

                Ok(new_property_id)
            }
        }
    }

    ///
    /// Retrieve the property ID for a database index
    ///
    pub (super) fn property_for_index(&mut self, property_index: i64) -> Result<CanvasPropertyId, CanvasError> {
        if let Some(cached_property) = self.property_for_id_cache.get(&property_index) {
            // We've encountered this index before so we know its property
            Ok(*cached_property)
        } else {
            // Fetch the property name from the database
            let mut query_name  = self.sqlite.prepare_cached("SELECT Name FROM Properties WHERE PropertyId = ?")?;
            let name            = query_name.query_one([property_index], |row| row.get::<_, String>(0))?;

            // Create the property ID and cache it
            let canvas_property_id = CanvasPropertyId::new(&name);
            self.property_id_cache.insert(canvas_property_id, property_index);
            self.property_for_id_cache.insert(property_index, canvas_property_id);

            Ok(canvas_property_id)
        }
    }

    ///
    /// Given a row returned from the database with the int, float and blob values for a property at the start, in
    /// that order, decodes the canvas property corresponding to that row.
    ///
    pub (super) fn decode_property(row: &Row) -> Option<CanvasProperty> {
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
    /// Sets the properties for a property target
    ///
    pub fn set_properties(&mut self, target: CanvasPropertyTarget, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
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
    pub fn delete_properties(&mut self, target: CanvasPropertyTarget, properties: Vec<CanvasPropertyId>) -> Result<(), CanvasError> {
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
    fn set_sql_properties<'a>(properties: impl Iterator<Item=(i64, &'a dyn ToSql)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), CanvasError> {
        for (property_idx, property) in properties {
            // Add the property ID and value to the parameters
            let mut params = other_params.clone();
            params.extend(params![property_idx, property]);

            let params: &[&dyn ToSql] = &params;

            // Run the query
            command.execute(params)?;
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
    fn delete_sql_properties(&mut self, command_template: &str, properties: impl Iterator<Item=i64>, other_params: Vec<&dyn ToSql>) -> Result<(), CanvasError> {
        // Delete all or nothing
        let transaction = self.sqlite.transaction()?;

        // Prepare the three commands to delete the three different types of property
        let delete_ints     = command_template.replace("{}", "Int");
        let delete_floats   = command_template.replace("{}", "Float");
        let delete_blobs    = command_template.replace("{}", "Blob");

        let mut delete_ints     = transaction.prepare_cached(&delete_ints)?;
        let mut delete_floats   = transaction.prepare_cached(&delete_floats)?;
        let mut delete_blobs    = transaction.prepare_cached(&delete_blobs)?;

        // Delete the properties in the list
        for property_idx in properties {
            // Append the property index to the params (so it's the last parameter)
            let mut params = other_params.clone();
            params.extend(params![property_idx]);
            let params: &[&dyn ToSql] = &params;

            // Delete from the three properties tables
            delete_ints.execute(params)?;
            delete_floats.execute(params)?;
            delete_blobs.execute(params)?;
        }

        // Finish up the transaction
        drop(delete_ints);
        drop(delete_floats);
        drop(delete_blobs);

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Sets any int properties found in the specified properties array. Property values are appended to the supplied default parameters
    ///
    pub (super) fn set_int_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), CanvasError> {
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
    pub (super) fn set_float_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), CanvasError> {
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
    pub (super) fn set_blob_properties(properties: &Vec<(i64, CanvasProperty)>, command: &mut CachedStatement<'_>, other_params: Vec<&dyn ToSql>) -> Result<(), CanvasError> {
        // Only set the blob properties that are requested
        let blob_properties = properties.iter()
            .filter_map(|(property_idx, property)| {
                match property {
                    CanvasProperty::Int(_)      | 
                    CanvasProperty::Float(_)    => None,
                    property                    => Some(postcard::to_allocvec(property).map(|val| (*property_idx, val)).map_err(|e| e.into()))
                }
            })
            .collect::<Result<Vec<_>, CanvasError>>()?;

        // Need references to the blobs we've built up
        let blob_properties = blob_properties.iter()
            .map::<(_, &dyn ToSql), _>(|(idx, prop)| (*idx, prop));

        Self::set_sql_properties(blob_properties, command, other_params)?;

        Ok(())
    }

    ///
    /// Updates the properties for a document
    ///
    pub fn set_document_properties(&mut self, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction()?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentIntProperties (PropertyId, IntValue) VALUES (?, ?)")?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentFloatProperties (PropertyId, FloatValue) VALUES (?, ?)")?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO DocumentBlobProperties (PropertyId, BlobValue) VALUES (?, ?)")?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![])?;
        }

        transaction.commit()?;

        Ok(())
    }
}
