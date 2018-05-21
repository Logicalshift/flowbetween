use animation::*;

use desync::*;
use rusqlite::*;

use std::mem;
use std::sync::*;

#[cfg(test)] mod tests;

mod flo_store;
mod flo_query;
mod flo_sqlite;
mod db_enum;
mod editlog;
mod insert_editlog;
mod animation;
mod animation_editor;
mod animation_core;
mod color;
mod brush;
mod vector_layer;
pub mod vector_frame;

pub use self::animation::*;
pub use self::insert_editlog::*;
pub use self::vector_layer::*;
use self::animation_editor::*;
use self::animation_core::*;
use self::flo_sqlite::*;
use self::flo_store::*;
use self::flo_query::*;

///
/// Database used to store an animation
/// 
pub struct AnimationDb {
    /// The core contains details of the database
    core: Arc<Desync<AnimationDbCore<FloSqlite>>>,

    /// The next available element ID
    next_element_id: Arc<Mutex<i64>>,

    /// The editor is used to provide the mutable animation interface (we keep it around so it can cache values if necessary)
    editor: Mutex<AnimationEditor<FloSqlite>>
}

impl AnimationDb {
    ///
    /// Creates a new animation database with an in-memory database
    /// 
    pub fn new() -> AnimationDb {
        Self::new_from_connection(Connection::open_in_memory().unwrap())
    }

    ///
    /// Creates a new animation database using the specified SQLite connection
    /// 
    pub fn new_from_connection(connection: Connection) -> AnimationDb {
        FloSqlite::setup(&connection).unwrap();

        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));
        let editor  = AnimationEditor::new(&core);

        // We begin assigning element IDs at the current length of the edit log
        let initial_element_id = core.sync(|core| core.db.query_edit_log_length()).unwrap() as i64;

        let db      = AnimationDb {
            core:               core,
            editor:             Mutex::new(editor),
            next_element_id:    Arc::new(Mutex::new(initial_element_id))
        };

        db
    }

    ///
    /// Creates an animation database that uses an existing database already set up in a SQLite connection
    /// 
    pub fn from_connection(connection: Connection) -> AnimationDb {
        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));
        let editor  = AnimationEditor::new(&core);

        // We begin assigning element IDs at the current length of the edit log
        let initial_element_id = core.sync(|core| core.db.query_edit_log_length()).unwrap() as i64;

        let db = AnimationDb {
            core:               core,
            editor:             Mutex::new(editor),
            next_element_id:    Arc::new(Mutex::new(initial_element_id))
        };

        db
    }

    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    pub fn retrieve_and_clear_error(&self) -> Option<Error> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        self.core.sync(|core| {
            core.retrieve_and_clear_error()
        })
    }

    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore<FloSqlite>) -> Result<()>>(&self, action: TFn) {
        self.core.async(move |core| {
            // Only run the function if there has been no failure
            if core.failure.is_none() {
                // Run the function and update the error status
                let result      = action(core);
                core.failure    = result.err();
            }
        })
    }

    ///
    /// Creates an animation editor
    /// 
    pub fn edit<'a>(&'a self) -> Editor<'a, MutableAnimation> {
        let editor: &Mutex<MutableAnimation> = &self.editor;
        let editor  = editor.lock().unwrap();

        Editor::new(editor)
    }

    ///
    /// Assigns a new, unique, element ID for the database
    /// 
    pub fn assign_element_id(&self) -> i64 {
        let mut next_element_id = self.next_element_id.lock().unwrap();
        let id                  = *next_element_id;

        *next_element_id += 1;
        id
    }
}

impl AnimationDbCore<FloSqlite> {
    ///
    /// Creates a new database core with a sqlite connection
    /// 
    fn new(connection: Connection) -> AnimationDbCore<FloSqlite> {
        let core = AnimationDbCore {
            db:             FloSqlite::new(connection),
            failure:        None
        };

        core
    }
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    fn retrieve_and_clear_error(&mut self) -> Option<Error> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        let mut failure = None;
        mem::swap(&mut self.failure, &mut failure);

        failure
    }
}
