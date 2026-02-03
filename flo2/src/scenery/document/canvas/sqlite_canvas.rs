use super::layer::*;
use super::queries::*;
use super::vector_editor::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use rusqlite::*;
use ::serde::*;

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
            Edit(AddLayer { new_layer_id, before_layer, })          => { canvas.add_layer(new_layer_id, before_layer); }
            Edit(RemoveLayer(layer_id))                             => { todo!() }
            Edit(ReorderLayer { layer_id, before_shape, })          => { todo!() }
            Edit(AddShape(shape_id, shape_defn))                    => { todo!() }
            Edit(RemoveShape(shape_id))                             => { todo!() }
            Edit(AddBrush(brush_id))                                => { todo!() }
            Edit(RemoveBrush(brush_id))                             => { todo!() }
            Edit(ReorderShape { shape_id, before_shape, })          => { todo!() }
            Edit(SetShapeParent(shape_id, parent))                  => { todo!() }
            Edit(SetProperty(property_target, properties))          => { todo!() }
            Edit(AddShapeBrushes(shape_id, brush_id))               => { todo!() }
            Edit(RemoveProperty(property_target, property_list))    => { todo!() }
            Edit(RemoveShapeBrushes(shape_id, brush_list))          => { todo!() }
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
    sqlite: Connection,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Self {
        Self { 
            sqlite
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

        let new_layer_idx = if let Some(before_layer) = before_layer {
            // Add between the existing layers
            let before_idx = transaction.query_one::<i64, _, _>("SELECT Idx FROM Layers WHERE LayerGuid = ?", [before_layer.to_string()], |row| row.get(0)).map_err(|_| ())?;
            transaction.execute("UPDATE Layers SET Idx = Idx + 1 WHERE Idx >= ?", [before_idx]).map_err(|_| ())?;

            before_idx
        } else {
            // Add the layer at the end
            let max_idx = transaction.query_one::<Option<i64>, _, _>("SELECT MAX(Idx) FROM Layers", [], |row| row.get(0)).map_err(|_| ())?;
            max_idx.map(|idx| idx + 1).unwrap_or(0)
        };

        // Add the layer itself
        transaction.execute("INSERT INTO Layers(LayerGuid, Idx) VALUES (?, ?)", params![new_layer_id.to_string(), new_layer_idx]).map_err(|_| ())?;

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
        let mut select_layers   = self.sqlite.prepare_cached("SELECT LayerGuid FROM Layers ORDER BY Idx ASC").map_err(|_| ())?;
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
}
