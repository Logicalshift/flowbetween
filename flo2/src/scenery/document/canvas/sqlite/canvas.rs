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
    pub (super) next_shape_id: Option<i64>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Result<Self, CanvasError> {
        sqlite.execute_batch("PRAGMA foreign_keys = ON")?;

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
    pub fn initialise(&self) -> Result<(), CanvasError> {
        self.sqlite.execute_batch(SCHEMA)?;

        Ok(())
    }

    ///
    /// Creates a new SQLite canvas in memory
    ///
    pub fn new_in_memory() -> Result<Self, CanvasError> {
        let sqlite  = Connection::open_in_memory()?;
        let canvas  = Self::with_connection(sqlite)?;
        canvas.initialise()?;

        Ok(canvas)
    }

    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_brush(&mut self, brush_id: CanvasBrushId) -> Result<i64, CanvasError> {
        Ok(self.sqlite.query_one::<i64, _, _>("SELECT BrushId FROM Brushes WHERE BrushGuid = ?", [brush_id.to_string()], |row| row.get(0))?)
    }

    ///
    /// Updates the properties for a brush
    ///
    pub fn set_brush_properties(&mut self, brush_id: CanvasBrushId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
        let brush_idx = self.index_for_brush(brush_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction()?;

        // Run commands to set each type of property value
        {
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushIntProperties (BrushId, PropertyId, IntValue) VALUES (?, ?, ?)")?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushFloatProperties (BrushId, PropertyId, FloatValue) VALUES (?, ?, ?)")?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushBlobProperties (BrushId, PropertyId, BlobValue) VALUES (?, ?, ?)")?;
            Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&brush_idx])?;
        }

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Sends a query response stored in a Vec<()>
    ///
    pub async fn send_vec_query_response(&mut self, target: StreamTarget, context: &SceneContext, generate_response: impl FnOnce(&mut SqliteCanvas, &mut Vec<VectorResponse>) -> Result<(), CanvasError>) -> Result<(), CanvasError> {
        // Connect to the query response target
        let mut target      = context.send(target)?;
        let mut response    = vec![];

        // Generate the values to send
        generate_response(self, &mut response)?;

        // Send the query response message
        target.send(QueryResponse::with_iterator(response)).await?;

        Ok(())
    }

    ///
    /// Changes the ordering of a layer
    ///
    pub fn reorder_layer(&mut self, layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>) -> Result<(), CanvasError> {
        let transaction = self.sqlite.transaction()?;

        // Work out the layer indexes where we want to add the new layer and the 
        let original_layer_order  = Self::order_for_layer_in_transaction(&transaction, layer_id)?;
        let before_layer_order    = if let Some(before_layer) = before_layer {
            Self::order_for_layer_in_transaction(&transaction, before_layer)?            
        } else {
            let max_order = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(OrderIdx) FROM Layers", [], |row| row.get(0))?;
            max_order.map(|idx| idx + 1).unwrap_or(0)
        };

        // Move the layers after the original layer
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx - 1 WHERE OrderIdx > ?", params![original_layer_order])?;
        let before_layer_order = if before_layer_order > original_layer_order {
            before_layer_order-1
        } else {
            before_layer_order
        };

        // Move the layers after the before layer index
        transaction.execute("UPDATE Layers SET OrderIdx = OrderIdx + 1 WHERE OrderIdx >= ?", params![before_layer_order])?;

        // Move the re-ordered layer to its new position
        transaction.execute("UPDATE Layers SET OrderIdx = ? WHERE LayerGuid = ?", params![before_layer_order, layer_id.to_string()])?;

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Adds a brush to the canvas
    ///
    pub fn add_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), CanvasError> {
        self.sqlite.execute("INSERT INTO Brushes (BrushGuid) VALUES (?)", params![brush_id.to_string()])?;

        Ok(())
    }

    ///
    /// Removes a brush and all its associations from the canvas
    ///
    pub fn remove_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), CanvasError> {
        self.sqlite.execute("DELETE FROM Brushes WHERE BrushGuid = ?", params![brush_id.to_string()])?;

        Ok(())
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
    pub fn query_document(&mut self, document_response: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
        // Read the properties from the three properties tables
        let mut q_int_properties    = self.sqlite.prepare_cached("SELECT IntValue,   PropertyId FROM DocumentIntProperties")?;
        let mut q_float_properties  = self.sqlite.prepare_cached("SELECT FloatValue, PropertyId FROM DocumentFloatProperties")?;
        let mut q_blob_properties   = self.sqlite.prepare_cached("SELECT BlobValue,  PropertyId FROM DocumentBlobProperties")?;

        let int_properties          = q_int_properties.query_map([],   |row| Ok((row.get::<_, i64>(1)?, CanvasProperty::Int(row.get(0)?))))?.collect::<Result<Vec<_>, _>>()?;
        let float_properties        = q_float_properties.query_map([], |row| Ok((row.get::<_, i64>(1)?, CanvasProperty::Float(row.get(0)?))))?.collect::<Result<Vec<_>, _>>()?;
        let blob_properties         = q_blob_properties.query_map([],  |row| Ok((row.get::<_, i64>(1)?, postcard::from_bytes::<CanvasProperty>(&row.get::<_, Vec<u8>>(0)?).ok())))?.collect::<Result<Vec<_>, _>>()?;

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
    pub fn query_layers(&mut self, query_layers: impl IntoIterator<Item=CanvasLayerId>, layer_response: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
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
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        // We query layers one at a time (would be nice if we could pass in an array to Sqlite)
        let mut layers = vec![];

        // Read the property values
        for layer in query_layers {
            // Read the properties for this layer
            let properties = properties_query.query_map(params![layer.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                })?
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
                .collect::<Result<Vec<_>, CanvasError>>()?;

            layer_response.push(VectorResponse::Layer(layer, properties));
        }

        Ok(())
    }

    ///
    /// Queries a list of brushes for their properties
    ///
    pub fn query_brushes(&mut self, query_brushes: impl IntoIterator<Item=CanvasBrushId>, brush_response: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
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
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        // We query brushes one at a time (would be nice if we could pass in an array to Sqlite)
        let mut brushes = vec![];

        // Read the property values
        for brush_id in query_brushes {
            // Read the properties for this brush
            let properties = properties_query.query_map(params![brush_id.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                })?
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
                .collect::<Result<Vec<_>, CanvasError>>()?;

            brush_response.push(VectorResponse::Brush(brush_id, properties));
        }

        Ok(())
    }

    ///
    /// Queries a list of shapes for their properties
    ///
    pub fn query_shapes(&mut self, query_shapes: impl IntoIterator<Item=CanvasShapeId>, shape_response: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
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
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        let mut shapes = vec![];

        // Read the property values
        for shape in query_shapes {
            // Read the properties for this shape
            let properties = properties_query.query_map(params![shape.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(3)?;
                    let property_value  = Self::decode_property(&row);

                    Ok((property_idx, property_value))
                })?
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
                .collect::<Result<Vec<_>, CanvasError>>()?;
            let shape_type = self.shapetype_for_shape(shape_id)?;

            shape_response.push(VectorResponse::Shape(shape_id, shape_type, properties));
        }

        Ok(())
    }

    ///
    /// Queries the shapes and their properties on a particular layer
    ///
    pub fn query_shapes_on_layer(&mut self, layer: CanvasLayerId, shape_response: &mut Vec<VectorResponse>, when: Duration) -> Result<(), CanvasError> {
        let latest_time_nanos   = when.as_nanos() as i64;
        let earliest_time_nanos = self.layer_frame_time(layer, when)?.as_nanos() as i64;

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
            WHERE l.LayerGuid = ?1 AND sl.Time >= ?3 AND sl.Time <= ?2

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
            WHERE l.LayerGuid = ?1 AND sl.Time >= ?3 AND sl.Time <= ?2

            ORDER BY ShapeOrder ASC, PropertyId ASC, Source ASC, BrushOrder DESC
            ";
        let mut shapes_query = self.sqlite.prepare_cached(shapes_query)?;

        // Shapes we've seen from the query, and the properties we've gathered from each shape
        let mut shapes      = vec![];
        let mut properties  = vec![];

        let mut shapes_rows      = shapes_query.query(params![layer.to_string(), latest_time_nanos, earliest_time_nanos])?;
        let mut cur_shape_idx    = None;
        let mut cur_shape_id     = None;
        let mut cur_shape_type   = None;
        let mut cur_property_idx = None;
        let mut cur_group        = None;

        while let Ok(Some(shape_row)) = shapes_rows.next() {
            // Update the shape that we're reading
            let shape_idx = shape_row.get::<_, i64>(7)?;
            let group_idx = shape_row.get::<_, Option<i64>>(9)?;
            if Some(shape_idx) != cur_shape_idx {
                // Finished receiving properties for the current shape, so move on to the next
                if let (Some(old_shape_id), Some(old_shape_idx), Some(old_shape_type)) = (cur_shape_id, cur_shape_idx, cur_shape_type) {
                    // cur_group contains the old group at this point
                    shapes.push((old_shape_idx, old_shape_id, old_shape_type, properties, cur_group));
                    properties = vec![];
                }

                // Update to track the shape indicated by the current row
                cur_shape_idx    = Some(shape_idx);
                cur_shape_id     = Some(CanvasShapeId::from_string(&shape_row.get::<_, String>(8)?));
                cur_shape_type   = Some(shape_row.get::<_, i64>(10)?);
                cur_property_idx = None;
                cur_group        = group_idx;
            }

            // Read the next property
            if let Some(property_value) = Self::decode_property(&shape_row) {
                let property_idx = shape_row.get::<_, i64>(3)?;

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
                .collect::<Result<Vec<_>, CanvasError>>()?;

            shape_response.push(VectorResponse::Shape(shape_id, shape_type, properties));

            last_group_idx  = group_idx;
            last_shape_idx  = Some(shape_idx);
        }

        Ok(())
    }

    ///
    /// Queries a list of layers, retrieving their properties and any shapes that are on them
    ///
    pub fn query_layers_with_shapes(&mut self, query_layers: impl IntoIterator<Item=CanvasLayerId>, layer_response: &mut Vec<VectorResponse>, when: Duration) -> Result<(), CanvasError> {
        // Query the layers
        let mut layers = vec![];
        self.query_layers(query_layers, &mut layers)?;

        // Merge in the shapes to generate the result
        for response in layers {
            match response {
                VectorResponse::Layer(layer_id, layer_properties) => {
                    layer_response.push(VectorResponse::Layer(layer_id, layer_properties));
                    self.query_shapes_on_layer(layer_id, layer_response, when)?;
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
    pub fn query_document_outline(&mut self, outline: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
        // Add the document properties to start
        self.query_document(outline)?;

        // Layers are fetched in order
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY OrderIdx ASC")?;
        let layers              = select_layers.query_map(params![], |row| Ok(CanvasLayerId::from_string(&row.get::<_, String>(0)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(select_layers);

        // Use the layer query to populate the layers, then write the order
        self.query_layers(layers.iter().copied(), outline)?;
        outline.push(VectorResponse::LayerOrder(layers));

        Ok(())
    }

    ///
    /// Queries the whole of the document
    ///
    pub fn query_document_whole(&mut self, outline: &mut Vec<VectorResponse>, when: Duration) -> Result<(), CanvasError> {
        // Add the document properties to start
        self.query_document(outline)?;

        // Layers are fetched in order
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY OrderIdx ASC")?;
        let layers              = select_layers.query_map(params![], |row| Ok(CanvasLayerId::from_string(&row.get::<_, String>(0)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(select_layers);

        // Use the layer query to populate the layers, then write the order
        self.query_layers_with_shapes(layers.iter().copied(), outline, when)?;
        outline.push(VectorResponse::LayerOrder(layers));

        // Query the brushes
        let mut select_brushes   = self.sqlite.prepare_cached("SELECT BrushGuid FROM Brushes")?;
        let brushes              = select_brushes.query_map(params![], |row| Ok(CanvasBrushId::from_string(&row.get::<_, String>(0)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(select_brushes);
        self.query_brushes(brushes, outline)?;

        Ok(())
    }
}
