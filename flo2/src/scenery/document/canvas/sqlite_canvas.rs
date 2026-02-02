use super::layer::*;
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
            Query(DocumentOutline(target))                                      => { todo!() },
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
    /// Adds a new layer to the canvas
    ///
    pub fn add_layer(&mut self, new_layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>) -> Result<(), ()> {
        let transaction = self.sqlite.transaction().map_err(|_| ())?;

        let new_layer_idx = if let Some(before_layer) = before_layer {
            // Add between the existing layers
            let before_idx = transaction.query_one::<i64, _, _>("SELECT Idx FROM Layers WHERE LayerGuid = ?", [before_layer.to_string()], |row| row.get(0)).map_err(|_| ())?;
            transaction.execute("UPDATE Layers SET Idx = Idx + 1 WHERE Idx > ?", [before_idx]).map_err(|_| ())?;

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

    #[test]
    fn add_layer() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        canvas.add_layer(CanvasLayerId::new(), None).unwrap();
    }

    #[test]
    fn add_two_layers() {
        let mut canvas = SqliteCanvas::new_in_memory().unwrap();
        canvas.add_layer(CanvasLayerId::new(), None).unwrap();
        canvas.add_layer(CanvasLayerId::new(), None).unwrap();
    }

    #[test]
    fn add_layer_before() {
        let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
        let first_layer     = CanvasLayerId::new();
        let second_layer    = CanvasLayerId::new();

        canvas.add_layer(first_layer, None).unwrap();
        canvas.add_layer(second_layer, Some(first_layer)).unwrap();
    }
}
