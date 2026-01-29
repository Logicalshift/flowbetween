use super::queries::*;
use super::vector_editor::*;

use flo_scene::*;

use rusqlite::*;
use ::serde::*;

/// Definition for the canvas sqlite storage
static SCHEMA: &'static str = include_str!("canvas.sql");

///
/// Messages for the sqlite canvas program
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SqliteCanvas {
    Edit(VectorCanvas),
    Query(VectorQuery),
}

///
/// Runs a program that edits the document stored in the Sqlite connection
///
pub async fn sqlite_canvas_program(input: InputStream<VectorCanvas>, context: SceneContext, connection: Connection) {
}

///
/// Edits a blank document in memory
///
pub async fn sqlite_canvas_program_new_in_memory(input: InputStream<VectorCanvas>, context: SceneContext) {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch(SCHEMA).unwrap();

    sqlite_canvas_program(input, context, connection).await;
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
}
