use super::canvas::*;
use super::super::brush::*;
use super::super::error::*;
use super::super::frame_time::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use rusqlite::*;

use std::result::{Result};

impl SqliteCanvas {
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
    /// Reads the document properties generate s a VectorResponse::Document in the response
    ///
    pub fn query_document(&mut self, document_response: &mut Vec<VectorResponse>) -> Result<(), CanvasError> {
        // Read the properties from the three properties tables
        let mut q_blob_properties   = self.sqlite.prepare_cached("SELECT BlobValue, PropertyId FROM DocumentBlobProperties")?;
        let blob_properties         = q_blob_properties.query_map([], |row| Ok((row.get::<_, i64>(1)?, postcard::from_bytes::<CanvasProperty>(&row.get::<_, Vec<u8>>(0)?).ok())))?.collect::<Result<Vec<_>, _>>()?;

        drop(q_blob_properties);

        // Combine them to create the final set of document properties
        let document_properties = 
            blob_properties.into_iter().flat_map(|(property_idx, maybe_value)| Some((property_idx, maybe_value?)))
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
        // Query the properties from the layer's blob properties table
        let properties_query =
            "
            SELECT          props.BlobValue, props.PropertyId
            FROM            Layers              l
            LEFT OUTER JOIN LayerBlobProperties props ON props.LayerId = l.LayerId
            WHERE           l.LayerGuid = ?
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        // We query layers one at a time (would be nice if we could pass in an array to Sqlite)
        let mut layers = vec![];

        // Read the property values
        for layer in query_layers {
            // Read the properties for this layer
            let properties = properties_query.query_map(params![layer.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(1)?;
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
        // Query to fetch the properties for each brush.
        let properties_query =
            "
            SELECT props.BlobValue, props.PropertyId
            FROM Brushes b
            LEFT OUTER JOIN BrushBlobProperties props ON props.BrushId = b.BrushId
            WHERE b.BrushGuid = ?
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        // We query brushes one at a time (would be nice if we could pass in an array to Sqlite)
        let mut brushes = vec![];

        // Read the property values
        for brush_id in query_brushes {
            // Read the properties for this brush
            let properties = properties_query.query_map(params![brush_id.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(1)?;
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
        // Query to fetch the properties for each shape, including brush properties from attached brushes.
        let properties_query =
            "
            SELECT          sp.BlobValue, sp.PropertyId, 0 AS Source, 0 AS BrushOrder
            FROM            Shapes              s
            LEFT OUTER JOIN ShapeBlobProperties sp ON sp.ShapeId = s.ShapeId
            WHERE           s.ShapeGuid = ?1

            UNION ALL

            SELECT          bp.BlobValue, bp.PropertyId, 1 AS Source, sb.OrderIdx AS BrushOrder
            FROM            Shapes              s
            INNER JOIN      ShapeBrushes        sb ON sb.ShapeId = s.ShapeId
            LEFT OUTER JOIN BrushBlobProperties bp ON bp.BrushId = sb.BrushId
            WHERE           s.ShapeGuid = ?1

            ORDER BY PropertyId ASC, Source ASC, BrushOrder DESC
            ";
        let mut properties_query = self.sqlite.prepare_cached(properties_query)?;

        let mut shapes = vec![];

        // Read the property values
        for shape in query_shapes {
            // Read the properties for this shape
            let properties = properties_query.query_map(params![shape.to_string()], |row| {
                    let property_idx    = row.get::<_, i64>(1)?;
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
            let shape_type   = self.shapetype_for_shape(shape_id)?;
            let canvas_shape = self.shape_for_shape_id(shape_id)?;
            let frame_time   = FrameTime::from_nanos(self.time_for_shape(shape_id)? as _);

            shape_response.push(VectorResponse::Shape(shape_id, canvas_shape, frame_time, shape_type, properties));
        }

        Ok(())
    }

    ///
    /// Queries the shapes and their properties on a particular layer
    ///
    pub fn query_shapes_on_layer(&mut self, layer: CanvasLayerId, shape_response: &mut Vec<VectorResponse>, when: FrameTime) -> Result<(), CanvasError> {
        let latest_time_nanos   = when.as_nanos();
        let earliest_time_nanos = self.layer_frame_time(layer, when)?.as_nanos();

        // Column indices for the shapes query result set
        const COL_PROPERTY_ID:      usize = 1;
        const COL_SHAPE_IDX:        usize = 5;
        const COL_SHAPE_GUID:       usize = 6;
        const COL_GROUP_IDX:        usize = 7;
        const COL_SHAPE_TYPE:       usize = 8;
        const COL_SHAPE_DATA_TYPE:  usize = 9;
        const COL_SHAPE_DATA:       usize = 10;
        const COL_FRAME_TIME:       usize = 11;

        // Query to fetch the properties for each shape, including brush properties from attached brushes.
        let shapes_query =
            "
            SELECT
                sp.BlobValue, sp.PropertyId,
                0 AS Source, 0 AS BrushOrder, sl.OrderIdx As ShapeOrder, s.ShapeId As ShapeIdx, s.ShapeGuid As ShapeGuid,
                g.ParentShapeId As GroupIdx, s.ShapeType AS ShapeType, s.ShapeDataType AS ShapeDataType, s.ShapeData AS ShapeData,
                sl.Time as FrameTime
            FROM            Shapes s
            INNER JOIN      ShapeLayers         sl  ON sl.ShapeId = s.ShapeId
            INNER JOIN      Layers              l   ON l.LayerId = sl.LayerId
            LEFT OUTER JOIN ShapeGroups         g   ON g.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeBlobProperties sp  ON sp.ShapeId = s.ShapeId
            WHERE           l.LayerGuid = ?1 AND sl.Time >= ?3 AND sl.Time <= ?2

            UNION ALL

            SELECT
                bp.BlobValue, bp.PropertyId,
                1 AS Source, sb.OrderIdx AS BrushOrder, sl.OrderIdx As ShapeOrder, s.ShapeId As ShapeIdx, s.ShapeGuid As ShapeGuid,
                g.ParentShapeId As GroupIdx, s.ShapeType AS ShapeType, s.ShapeDataType AS ShapeDataType, s.ShapeData AS ShapeData,
                sl.Time as FrameTime
            FROM Shapes s
            INNER JOIN      ShapeLayers         sl  ON sl.ShapeId = s.ShapeId
            INNER JOIN      Layers              l   ON l.LayerId = sl.LayerId
            INNER JOIN      ShapeBrushes        sb  ON sb.ShapeId = s.ShapeId
            LEFT OUTER JOIN ShapeGroups         g   ON g.ShapeId = s.ShapeId
            LEFT OUTER JOIN BrushBlobProperties bp  ON bp.BrushId = sb.BrushId
            WHERE l.LayerGuid = ?1 AND sl.Time >= ?3 AND sl.Time <= ?2

            ORDER BY ShapeOrder ASC, PropertyId ASC, Source ASC, BrushOrder DESC
            ";
        let mut shapes_query = self.sqlite.prepare_cached(shapes_query)?;

        // Shapes we've seen from the query, and the properties we've gathered from each shape
        let mut shapes      = vec![];
        let mut properties  = vec![];

        let mut shapes_rows          = shapes_query.query(params![layer.to_string(), latest_time_nanos, earliest_time_nanos])?;
        let mut cur_shape_idx        = None;
        let mut cur_shape_id         = None;
        let mut cur_shape_type       = None;
        let mut cur_shape_data_type  = None;
        let mut cur_shape_data       = None;
        let mut cur_property_idx     = None;
        let mut cur_group            = None;
        let mut cur_shape_time       = None;

        while let Ok(Some(shape_row)) = shapes_rows.next() {
            // Update the shape that we're reading
            let shape_idx = shape_row.get::<_, i64>(COL_SHAPE_IDX)?;
            let group_idx = shape_row.get::<_, Option<i64>>(COL_GROUP_IDX)?;
            if Some(shape_idx) != cur_shape_idx {
                // Finished receiving properties for the current shape, so move on to the next
                if let (Some(old_shape_id), Some(old_shape_idx), Some(old_shape_type), Some(old_data_type), Some(old_data), Some(old_time)) = (cur_shape_id, cur_shape_idx, cur_shape_type, cur_shape_data_type, cur_shape_data, cur_shape_time) {
                    // cur_group contains the old group at this point
                    shapes.push((old_shape_idx, old_shape_id, old_shape_type, old_data_type, old_data, properties, cur_group, old_time));
                    properties = vec![];
                }

                // Update to track the shape indicated by the current row
                cur_shape_idx       = Some(shape_idx);
                cur_shape_id        = Some(CanvasShapeId::from_string(&shape_row.get::<_, String>(COL_SHAPE_GUID)?));
                cur_shape_type      = Some(shape_row.get::<_, i64>(COL_SHAPE_TYPE)?);
                cur_shape_data_type = Some(shape_row.get::<_, i64>(COL_SHAPE_DATA_TYPE)?);
                cur_shape_data      = Some(shape_row.get::<_, Vec<u8>>(COL_SHAPE_DATA)?);
                cur_shape_time      = Some(shape_row.get::<_, i64>(COL_FRAME_TIME)?);
                cur_property_idx    = None;
                cur_group           = group_idx;
            }

            // Read the next property
            if let Some(property_value) = Self::decode_property(&shape_row) {
                let property_idx = shape_row.get::<_, i64>(COL_PROPERTY_ID)?;

                if Some(property_idx) != cur_property_idx {
                    // Only write the first property if the property is defined in more than one place
                    cur_property_idx = Some(property_idx);
                    properties.push((property_idx, property_value));
                }
            }
        }

        // Set the last shape
        if let (Some(shape_id), Some(shape_idx), Some(shape_type), Some(data_type), Some(data), Some(time)) = (cur_shape_id, cur_shape_idx, cur_shape_type, cur_shape_data_type, cur_shape_data, cur_shape_time) {
            shapes.push((shape_idx, shape_id, shape_type, data_type, data, properties, cur_group, time));
        }

        drop(shapes_rows);
        drop(shapes_query);

        // Generate the shapes for the response
        let mut last_group_idx  = None;
        let mut last_shape_idx  = None;
        let mut group_stack     = vec![];

        for (shape_idx, shape_id, shape_type_idx, shape_data_type, shape_data, properties, group_idx, frame_time) in shapes.into_iter() {
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
            let canvas_shape = Self::decode_shape(shape_data_type, &shape_data)?;
            let shape_type   = self.shapetype_for_index(shape_type_idx)?;
            let frame_time   = FrameTime::from_nanos(frame_time as _);
            let properties   = properties.into_iter()
                .map(|(property_idx, value)| Ok((self.property_for_index(property_idx)?, value)))
                .collect::<Result<Vec<_>, CanvasError>>()?;

            shape_response.push(VectorResponse::Shape(shape_id, canvas_shape, frame_time, shape_type, properties));

            last_group_idx  = group_idx;
            last_shape_idx  = Some(shape_idx);
        }

        Ok(())
    }

    ///
    /// Queries a list of layers, retrieving their properties and any shapes that are on them
    ///
    pub fn query_layers_with_shapes(&mut self, query_layers: impl IntoIterator<Item=CanvasLayerId>, layer_response: &mut Vec<VectorResponse>, when: FrameTime) -> Result<(), CanvasError> {
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
    pub fn query_document_whole(&mut self, outline: &mut Vec<VectorResponse>, when: FrameTime) -> Result<(), CanvasError> {
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
