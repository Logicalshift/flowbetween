use super::queries::*;
use super::vector_editor::*;

use flo_scene::*;

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
    let mut input = input;
    while let Some(msg) = input.next().await {
        use SqliteCanvasRequest::*;
        use VectorCanvas::*;
        use VectorQuery::*;

        match msg {
            Edit(AddLayer { new_layer_id, before_layer, })          => { todo!() }
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

            Query(WholeDocument)                                        => { todo!() },
            Query(DocumentOutline)                                      => { todo!() },
            Query(Layers(layer_list))                                   => { todo!() },
            Query(Shapes(shape_list))                                   => { todo!() },
            Query(Brushes(brush_list))                                  => { todo!() },
            Query(ShapesInRegion { search_layers, region, inclusive, }) => { todo!() },
            Query(ShapesAtPoint { search_layers, point, })              => { todo!() },
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

}

///
/// Storage for the sqlite canvas
///
#[derive(Clone)]
pub struct SqliteCanvas {
    sqlite: Arc<Connection>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Self {
        Self { 
            sqlite: Arc::new(sqlite)
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
}

#[cfg(test)]
mod test {
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
}
