use super::brush::*;
use super::layer::*;
use super::property::*;
use super::queries::*;
use super::shape::*;
use super::vector_editor::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use rusqlite::*;
use ::serde::*;

use std::collections::{HashMap};
use std::result::{Result};
use std::sync::*;

/// Definition for the canvas sqlite storage
static SCHEMA: &'static str = include_str!("canvas.sql");

///
/// Messages for the sqlite canvas program
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SqliteCanvasRequest {
    Edit(VectorCanvas),
    Query(VectorQuery),
}

///
/// Runs a program that edits the document stored in the Sqlite connection
///
pub async fn sqlite_canvas_program(input: InputStream<SqliteCanvasRequest>, context: SceneContext, canvas: SqliteCanvas) {
    let mut canvas  = canvas;
    let mut input   = input;
    while let Some(msg) = input.next().await {
        use SqliteCanvasRequest::*;
        use VectorCanvas::*;
        use VectorQuery::*;

        match msg {
            Edit(AddLayer { new_layer_id, before_layer, })          => { canvas.add_layer(new_layer_id, before_layer).ok(); }
            Edit(RemoveLayer(layer_id))                             => { canvas.remove_layer(layer_id).ok(); }
            Edit(ReorderLayer { layer_id, before_layer, })          => { canvas.reorder_layer(layer_id, before_layer).ok(); }
            Edit(AddShape(shape_id, shape_defn))                    => { canvas.add_shape(shape_id, shape_defn).ok(); }
            Edit(RemoveShape(shape_id))                             => { canvas.remove_shape(shape_id).ok(); }
            Edit(SetShapeDefinition(shape_id, shape_defn))          => { canvas.set_shape_definition(shape_id, shape_defn).ok(); }
            Edit(AddBrush(brush_id))                                => { todo!() }
            Edit(RemoveBrush(brush_id))                             => { todo!() }
            Edit(ReorderShape { shape_id, before_shape, })          => { canvas.reorder_shape(shape_id, before_shape).ok(); }
            Edit(SetShapeParent(shape_id, parent))                  => { canvas.set_shape_parent(shape_id, parent).ok(); }
            Edit(SetProperty(property_target, properties))          => { canvas.set_properties(property_target, properties).ok(); }
            Edit(AddShapeBrushes(shape_id, brush_ids))              => { canvas.add_shape_brushes(shape_id, brush_ids).ok(); }
            Edit(RemoveProperty(property_target, property_list))    => { todo!() }
            Edit(RemoveShapeBrushes(shape_id, brush_ids))           => { canvas.remove_shape_brushes(shape_id, brush_ids).ok(); }

            Edit(Subscribe(edit_target))                            => { todo!() }

            Query(WholeDocument(target))                                        => { todo!() },
            Query(DocumentOutline(target))                                      => { canvas.send_vec_query_response(target, &context, |canvas, response| canvas.query_document_outline(response)).await.ok(); },
            Query(Layers(target, layer_list))                                   => { todo!() },
            Query(Shapes(target, shape_list))                                   => { todo!() },
            Query(Brushes(target, brush_list))                                  => { todo!() },
            Query(ShapesInRegion { target, search_layers, region, inclusive, }) => { todo!() },
            Query(ShapesAtPoint { target, search_layers, point, })              => { todo!() },
        }
    }
}

///
/// Edits a blank document in memory
///
pub async fn sqlite_canvas_program_new_in_memory(input: InputStream<SqliteCanvasRequest>, context: SceneContext) {
    let canvas = SqliteCanvas::new_in_memory().unwrap();

    sqlite_canvas_program(input, context, canvas).await;
}

impl SceneMessage for SqliteCanvasRequest {
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::sqlite_canvas").into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.add_subprogram(SubProgramId::called("flowbetween::sqlite_canvas"), sqlite_canvas_program_new_in_memory, 20);

        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Edit(msg)))), (), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg)))), (), StreamId::with_message_type::<VectorQuery>()).unwrap();

        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Edit(msg))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorQuery>()).unwrap();
    }
}

///
/// Storage for the sqlite canvas
///
pub struct SqliteCanvas {
    /// Connection to the sqlite database
    sqlite: Connection,

    /// Cache of the known property IDs
    property_id_cache: HashMap<CanvasPropertyId, i64>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Self {
        Self { 
            sqlite:             sqlite,
            property_id_cache:  HashMap::new(),
        }
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
        let sqlite = Connection::open_in_memory().map_err(|_| ())?;
        let canvas = Self::with_connection(sqlite);
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
    fn index_for_property(&mut self, canvas_property_id: CanvasPropertyId) -> Result<i64, ()> {
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
            let mut int_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushIntProperties (ShapeId, PropertyId, IntValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_int_properties(&properties, &mut int_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut float_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushFloatProperties (ShapeId, PropertyId, FloatValue) VALUES (?, ?, ?)").map_err(|_| ())?;
            Self::set_float_properties(&properties, &mut float_properties_cmd, vec![&brush_idx])?;
        }

        {
            let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushBlobProperties (ShapeId, PropertyId, BlobValue) VALUES (?, ?, ?)").map_err(|_| ())?;
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

        // Remove from layer parent and compact ordering
        if let Ok((layer_id, order_idx)) = transaction.query_one("SELECT LayerId, OrderIdx FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            transaction.execute("DELETE FROM ShapeLayers WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeLayers SET OrderIdx = OrderIdx - 1 WHERE LayerId = ? AND OrderIdx > ?", params![layer_id, order_idx]).map_err(|_| ())?;
        }

        // Remove from group parent and compact ordering
        if let Ok((parent_id, order_idx)) = transaction.query_one("SELECT ParentShapeId, OrderIdx FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))) {
            transaction.execute("DELETE FROM ShapeGroups WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
            transaction.execute("UPDATE ShapeGroups SET OrderIdx = OrderIdx - 1 WHERE ParentShapeId = ? AND OrderIdx > ?", params![parent_id, order_idx]).map_err(|_| ())?;
        }

        // Detach any child shapes grouped under this shape
        transaction.execute("DELETE FROM ShapeGroups WHERE ParentShapeId = ?", params![shape_idx]).map_err(|_| ())?;

        // Remove brush associations
        transaction.execute("DELETE FROM ShapeBrushes WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;

        // Remove shape properties
        transaction.execute("DELETE FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
        transaction.execute("DELETE FROM ShapeFloatProperties WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;
        transaction.execute("DELETE FROM ShapeBlobProperties WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;

        // Remove the shape itself
        transaction.execute("DELETE FROM Shapes WHERE ShapeId = ?", params![shape_idx]).map_err(|_| ())?;

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

#[cfg(test)]
mod test {
    use flo_scene::commands::ReadCommand;

    use super::*;
    use super::super::point::*;

    /// Helper: returns the ShapeGuids for shapes on a layer, ordered by OrderIdx
    fn shapes_on_layer(canvas: &SqliteCanvas, layer_id: CanvasLayerId) -> Vec<String> {
        let layer_idx   = canvas.sqlite.query_one::<i64, _, _>("SELECT LayerId FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).unwrap();
        let mut stmt    = canvas.sqlite.prepare("SELECT s.ShapeGuid FROM ShapeLayers sl JOIN Shapes s ON sl.ShapeId = s.ShapeId WHERE sl.LayerId = ? ORDER BY sl.OrderIdx ASC").unwrap();
        let rows        = stmt.query_map(params![layer_idx], |row| row.get::<_, String>(0)).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    /// Helper: returns the ShapeGuids for shapes in a group, ordered by OrderIdx
    fn shapes_in_group(canvas: &SqliteCanvas, parent_shape_id: CanvasShapeId) -> Vec<String> {
        let parent_idx  = canvas.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [parent_shape_id.to_string()], |row| row.get(0)).unwrap();
        let mut stmt    = canvas.sqlite.prepare("SELECT s.ShapeGuid FROM ShapeGroups sg JOIN Shapes s ON sg.ShapeId = s.ShapeId WHERE sg.ParentShapeId = ? ORDER BY sg.OrderIdx ASC").unwrap();
        let rows        = stmt.query_map(params![parent_idx], |row| row.get::<_, String>(0)).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    /// Helper: returns the BrushGuids associated with a shape, ordered by OrderIdx
    fn brushes_on_shape(canvas: &SqliteCanvas, shape_id: CanvasShapeId) -> Vec<String> {
        let shape_idx   = canvas.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [shape_id.to_string()], |row| row.get(0)).unwrap();
        let mut stmt    = canvas.sqlite.prepare("SELECT b.BrushGuid FROM ShapeBrushes sb JOIN Brushes b ON sb.BrushId = b.BrushId WHERE sb.ShapeId = ? ORDER BY sb.OrderIdx ASC").unwrap();
        let rows        = stmt.query_map(params![shape_idx], |row| row.get::<_, String>(0)).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    /// Helper: directly inserts a brush into the Brushes table (since AddBrush is not yet implemented)
    fn insert_brush(canvas: &SqliteCanvas, brush_id: CanvasBrushId) {
        canvas.sqlite.execute("INSERT INTO Brushes (BrushGuid) VALUES (?)", params![brush_id.to_string()]).unwrap();
    }

    fn test_rect() -> CanvasShape {
        CanvasShape::Rectangle(CanvasRectangle { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 10.0, y: 10.0 } })
    }

    fn test_ellipse() -> CanvasShape {
        CanvasShape::Ellipse(CanvasEllipse { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 5.0, y: 5.0 }, direction: CanvasPoint { x: 1.0, y: 0.0 } })
    }

    #[test]
    fn initialize_schema() {
        // Should be able to initialize the database
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
    }

    #[test]
    fn initialise_canvas() {
        SqliteCanvas::new_in_memory().unwrap();
    }

    #[test]
    fn add_layer() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        canvas.add_layer(CanvasLayerId::new(), None).unwrap();
    }

    #[test]
    fn add_two_layers() {
        let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
        let first_layer     = CanvasLayerId::new();
        let second_layer    = CanvasLayerId::new();

        canvas.add_layer(first_layer, None).unwrap();
        canvas.add_layer(second_layer, None).unwrap();

        let mut layers = vec![];
        canvas.query_document_outline(&mut layers).unwrap();
        assert!(layers == vec![
            VectorResponse::Document(vec![]),
            VectorResponse::Layer(first_layer, vec![]), 
            VectorResponse::Layer(second_layer, vec![]),
            VectorResponse::LayerOrder(vec![first_layer, second_layer]), 
        ], "{:?} ({:?} {:?})", layers, first_layer, second_layer);
    }

    #[test]
    fn add_layer_before() {
        let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
        let first_layer     = CanvasLayerId::new();
        let second_layer    = CanvasLayerId::new();

        canvas.add_layer(first_layer, None).unwrap();
        canvas.add_layer(second_layer, Some(first_layer)).unwrap();

        let mut layers = vec![];
        canvas.query_document_outline(&mut layers).unwrap();
        assert!(layers == vec![
            VectorResponse::Document(vec![]),
            VectorResponse::Layer(second_layer, vec![]), 
            VectorResponse::Layer(first_layer, vec![]),
            VectorResponse::LayerOrder(vec![second_layer, first_layer]), 
        ], "{:?} ({:?} {:?})", layers, first_layer, second_layer);
    }

    #[test]
    fn query_document_outline() {
        let scene = Scene::default();

        #[derive(PartialEq, Debug, Serialize, Deserialize)]
        struct TestResponse(Vec<VectorResponse>);

        impl SceneMessage for TestResponse { }

        let test_program    = SubProgramId::new();
        let query_program   = SubProgramId::new();

        let layer_1         = CanvasLayerId::new();
        let layer_2         = CanvasLayerId::new();

        // Program that adds some layers and sends a test response
        scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
            let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
            let mut canvas  = context.send(()).unwrap();

            // Set up some layers (layer2 vs layer1)
            canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_1, before_layer: None }).await.unwrap();
            canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_2, before_layer: Some(layer_1) }).await.unwrap();

            // Query the document outline
            let outline = context.spawn_query(ReadCommand::default(), VectorQuery::DocumentOutline(().into()), ()).unwrap();
            let outline = outline.collect::<Vec<_>>().await;

            context.send_message(TestResponse(outline)).await.unwrap();
        }, 1);

        // The expected response to the query after this set up
        let expected = vec![
            VectorResponse::Document(vec![]),
            VectorResponse::Layer(layer_2, vec![]), 
            VectorResponse::Layer(layer_1, vec![]),
            VectorResponse::LayerOrder(vec![layer_2, layer_1]), 
        ];

        // Run the test
        TestBuilder::new()
            .expect_message_matching(TestResponse(expected), format!("Layer 1 = {:?}, layer 2 = {:?}", layer_1, layer_2))
            .run_in_scene(&scene, test_program);
    }

    #[test]
    fn set_property_ids() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();

        let property_1 = canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
        let property_2 = canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();

        assert!(property_1 == 1, "Property 1: {:?} != 1", property_1);
        assert!(property_2 == 2, "Property 2: {:?} != 2", property_2);
    }

    #[test]
    fn read_property_ids_from_cache() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();

        // Write some properties
        canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
        canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();

        // Clear the cache
        canvas.property_id_cache.clear();

        // Re-fetch the properties
        let property_1 = canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
        let property_2 = canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();
        let property_3 = canvas.index_for_property(CanvasPropertyId::new("Three")).unwrap();

        assert!(property_1 == 1, "Property 1: {:?} != 1", property_1);
        assert!(property_2 == 2, "Property 2: {:?} != 2", property_2);
        assert!(property_3 == 3, "Property 3: {:?} != 3", property_3);
    }

    #[test]
    fn set_document_properties() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();

        // Set some properties for the document
        canvas.set_document_properties(vec![
            (CanvasPropertyId::new("One"), CanvasProperty::Int(42)),
            (CanvasPropertyId::new("Two"), CanvasProperty::Float(42.0)),
            (CanvasPropertyId::new("Three"), CanvasProperty::IntList(vec![1, 2, 3])),
        ]).unwrap();
    }

    #[test]
    fn set_layer_properties() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();

        // Create a layer
        let layer = CanvasLayerId::new();
        canvas.add_layer(layer, None).unwrap();

        // Set some properties for the layer
        canvas.set_layer_properties(layer, vec![
            (CanvasPropertyId::new("One"), CanvasProperty::Int(42)),
            (CanvasPropertyId::new("Two"), CanvasProperty::Float(42.0)),
            (CanvasPropertyId::new("Three"), CanvasProperty::IntList(vec![1, 2, 3])),
        ]).unwrap();
    }

    #[test]
    fn add_shape() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        let shape = CanvasShapeId::new();

        canvas.add_shape(shape, test_rect()).unwrap();

        // Shape should exist in the database
        assert!(canvas.index_for_shape(shape).is_ok());
    }

    #[test]
    fn add_shape_replaces_existing() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        let shape = CanvasShapeId::new();

        canvas.add_shape(shape, test_rect()).unwrap();
        let idx_before = canvas.index_for_shape(shape).unwrap();

        // Adding the same shape ID again should replace in place
        canvas.add_shape(shape, test_ellipse()).unwrap();
        let idx_after = canvas.index_for_shape(shape).unwrap();

        assert!(idx_before == idx_after, "ShapeId should be preserved on replace");

        // Verify the type was updated
        let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![idx_after], |row| row.get(0)).unwrap();
        assert!(shape_type == CANVAS_ELLIPSE_V1_TYPE, "Shape type should be ellipse ({}), got {}", CANVAS_ELLIPSE_V1_TYPE, shape_type);
    }

    #[test]
    fn remove_shape() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer       = CanvasLayerId::new();
        let shape       = CanvasShapeId::new();

        canvas.add_layer(layer, None).unwrap();
        canvas.add_shape(shape, test_rect()).unwrap();
        canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();

        // Shape should be on the layer
        assert!(shapes_on_layer(&canvas, layer).len() == 1);

        canvas.remove_shape(shape).unwrap();

        // Shape should be gone from both Shapes and ShapeLayers
        assert!(canvas.index_for_shape(shape).is_err());
        assert!(shapes_on_layer(&canvas, layer).is_empty());
    }

    #[test]
    fn remove_group_detaches_children() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let group       = CanvasShapeId::new();
        let child       = CanvasShapeId::new();

        canvas.add_shape(group, CanvasShape::Group).unwrap();
        canvas.add_shape(child, test_rect()).unwrap();
        canvas.set_shape_parent(child, CanvasShapeParent::Shape(group)).unwrap();
        assert!(shapes_in_group(&canvas, group).len() == 1);

        // Removing the group should detach the child
        canvas.remove_shape(group).unwrap();
        assert!(canvas.index_for_shape(child).is_ok(), "Child should still exist");
        assert!(canvas.index_for_shape(group).is_err(), "Group should be removed");

        // Child should no longer be in any group
        let child_idx       = canvas.index_for_shape(child).unwrap();
        let in_any_group    = canvas.sqlite.query_one::<i64, _, _>("SELECT COUNT(*) FROM ShapeGroups WHERE ShapeId = ?", params![child_idx], |row| row.get(0)).unwrap();
        assert!(in_any_group == 0, "Child should not be in any group after parent is removed");
    }

    #[test]
    fn set_shape_definition() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        let shape = CanvasShapeId::new();

        canvas.add_shape(shape, test_rect()).unwrap();
        let shape_idx = canvas.index_for_shape(shape).unwrap();

        // Check initial type
        let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
        assert!(shape_type == CANVAS_RECTANGLE_V1_TYPE);

        // Replace definition with an ellipse
        canvas.set_shape_definition(shape, test_ellipse()).unwrap();
        let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
        assert!(shape_type == CANVAS_ELLIPSE_V1_TYPE);
    }

    #[test]
    fn set_shape_parent_to_layer() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer       = CanvasLayerId::new();
        let shape       = CanvasShapeId::new();

        canvas.add_layer(layer, None).unwrap();
        canvas.add_shape(shape, test_rect()).unwrap();

        // Initially not on any layer
        assert!(shapes_on_layer(&canvas, layer).is_empty());

        canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();
        assert!(shapes_on_layer(&canvas, layer) == vec![shape.to_string()]);
    }

    #[test]
    fn set_shape_parent_to_group() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer       = CanvasLayerId::new();
        let group       = CanvasShapeId::new();
        let child       = CanvasShapeId::new();

        canvas.add_layer(layer, None).unwrap();
        canvas.add_shape(group, CanvasShape::Group).unwrap();
        canvas.add_shape(child, test_rect()).unwrap();

        // Put shape on a layer first, then move to group
        canvas.set_shape_parent(child, CanvasShapeParent::Layer(layer)).unwrap();
        assert!(shapes_on_layer(&canvas, layer) == vec![child.to_string()]);

        canvas.set_shape_parent(child, CanvasShapeParent::Shape(group)).unwrap();

        // Should be removed from the layer and added to the group
        assert!(shapes_on_layer(&canvas, layer).is_empty());
        assert!(shapes_in_group(&canvas, group) == vec![child.to_string()]);
    }

    #[test]
    fn set_shape_parent_detach() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer       = CanvasLayerId::new();
        let shape       = CanvasShapeId::new();

        canvas.add_layer(layer, None).unwrap();
        canvas.add_shape(shape, test_rect()).unwrap();
        canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();
        assert!(shapes_on_layer(&canvas, layer).len() == 1);

        canvas.set_shape_parent(shape, CanvasShapeParent::None).unwrap();
        assert!(shapes_on_layer(&canvas, layer).is_empty());
        assert!(canvas.index_for_shape(shape).is_ok(), "Shape should still exist after detach");
    }

    #[test]
    fn reorder_shape_on_layer() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer       = CanvasLayerId::new();
        let shape_a     = CanvasShapeId::new();
        let shape_b     = CanvasShapeId::new();
        let shape_c     = CanvasShapeId::new();

        canvas.add_layer(layer, None).unwrap();
        canvas.add_shape(shape_a, test_rect()).unwrap();
        canvas.add_shape(shape_b, test_rect()).unwrap();
        canvas.add_shape(shape_c, test_rect()).unwrap();
        canvas.set_shape_parent(shape_a, CanvasShapeParent::Layer(layer)).unwrap();
        canvas.set_shape_parent(shape_b, CanvasShapeParent::Layer(layer)).unwrap();
        canvas.set_shape_parent(shape_c, CanvasShapeParent::Layer(layer)).unwrap();

        // Order is A, B, C
        assert!(shapes_on_layer(&canvas, layer) == vec![shape_a.to_string(), shape_b.to_string(), shape_c.to_string()]);

        // Move C before A -> C, A, B
        canvas.reorder_shape(shape_c, Some(shape_a)).unwrap();
        assert!(shapes_on_layer(&canvas, layer) == vec![shape_c.to_string(), shape_a.to_string(), shape_b.to_string()]);

        // Move A to end (before = None) -> C, B, A
        canvas.reorder_shape(shape_a, None).unwrap();
        assert!(shapes_on_layer(&canvas, layer) == vec![shape_c.to_string(), shape_b.to_string(), shape_a.to_string()]);
    }

    #[test]
    fn reorder_shape_in_group() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let group       = CanvasShapeId::new();
        let shape_a     = CanvasShapeId::new();
        let shape_b     = CanvasShapeId::new();

        canvas.add_shape(group, CanvasShape::Group).unwrap();
        canvas.add_shape(shape_a, test_rect()).unwrap();
        canvas.add_shape(shape_b, test_rect()).unwrap();
        canvas.set_shape_parent(shape_a, CanvasShapeParent::Shape(group)).unwrap();
        canvas.set_shape_parent(shape_b, CanvasShapeParent::Shape(group)).unwrap();

        // Order is A, B
        assert!(shapes_in_group(&canvas, group) == vec![shape_a.to_string(), shape_b.to_string()]);

        // Move B before A -> B, A
        canvas.reorder_shape(shape_b, Some(shape_a)).unwrap();
        assert!(shapes_in_group(&canvas, group) == vec![shape_b.to_string(), shape_a.to_string()]);
    }

    #[test]
    fn reorder_shape_different_parent_is_error() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let layer_1     = CanvasLayerId::new();
        let layer_2     = CanvasLayerId::new();
        let shape_a     = CanvasShapeId::new();
        let shape_b     = CanvasShapeId::new();

        canvas.add_layer(layer_1, None).unwrap();
        canvas.add_layer(layer_2, None).unwrap();
        canvas.add_shape(shape_a, test_rect()).unwrap();
        canvas.add_shape(shape_b, test_rect()).unwrap();
        canvas.set_shape_parent(shape_a, CanvasShapeParent::Layer(layer_1)).unwrap();
        canvas.set_shape_parent(shape_b, CanvasShapeParent::Layer(layer_2)).unwrap();

        // Trying to reorder shape_a before shape_b should fail because they have different parents
        let result = canvas.reorder_shape(shape_a, Some(shape_b));
        assert!(result.is_err(), "Reordering across different parents should fail (re-parent first)");

        // Shapes should be unchanged
        assert!(shapes_on_layer(&canvas, layer_1) == vec![shape_a.to_string()]);
        assert!(shapes_on_layer(&canvas, layer_2) == vec![shape_b.to_string()]);
    }

    #[test]
    fn reorder_unparented_shape_is_error() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let shape       = CanvasShapeId::new();

        canvas.add_shape(shape, test_rect()).unwrap();

        // Shape has no parent, reorder should fail
        let result = canvas.reorder_shape(shape, None);
        assert!(result.is_err(), "Reordering a shape with no parent should fail");
    }

    #[test]
    fn add_shape_brushes() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let shape       = CanvasShapeId::new();
        let brush_1     = CanvasBrushId::new();
        let brush_2     = CanvasBrushId::new();

        canvas.add_shape(shape, test_rect()).unwrap();
        insert_brush(&canvas, brush_1);
        insert_brush(&canvas, brush_2);

        canvas.add_shape_brushes(shape, vec![brush_1, brush_2]).unwrap();
        assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_2.to_string()]);
    }

    #[test]
    fn add_shape_brushes_appends() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let shape       = CanvasShapeId::new();
        let brush_1     = CanvasBrushId::new();
        let brush_2     = CanvasBrushId::new();
        let brush_3     = CanvasBrushId::new();

        canvas.add_shape(shape, test_rect()).unwrap();
        insert_brush(&canvas, brush_1);
        insert_brush(&canvas, brush_2);
        insert_brush(&canvas, brush_3);

        canvas.add_shape_brushes(shape, vec![brush_1]).unwrap();
        canvas.add_shape_brushes(shape, vec![brush_2, brush_3]).unwrap();
        assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_2.to_string(), brush_3.to_string()]);
    }

    #[test]
    fn remove_shape_brushes() {
        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
        let shape       = CanvasShapeId::new();
        let brush_1     = CanvasBrushId::new();
        let brush_2     = CanvasBrushId::new();
        let brush_3     = CanvasBrushId::new();

        canvas.add_shape(shape, test_rect()).unwrap();
        insert_brush(&canvas, brush_1);
        insert_brush(&canvas, brush_2);
        insert_brush(&canvas, brush_3);

        canvas.add_shape_brushes(shape, vec![brush_1, brush_2, brush_3]).unwrap();
        canvas.remove_shape_brushes(shape, vec![brush_2]).unwrap();

        // brush_1 and brush_3 should remain in order
        assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_3.to_string()]);
    }
}
