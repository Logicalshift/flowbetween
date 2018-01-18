use animation::*;

use desync::*;
use rusqlite::*;

use std::mem;
use std::sync::*;

#[cfg(test)] mod tests;

mod flo_sqlite;
mod db_enum;
mod db_update;
mod editlog;
mod animation;
mod mutable_animation;
mod core;
mod color;
mod brush;
mod vector_layer;

pub use self::animation::*;
pub use self::editlog::*;
pub use self::vector_layer::*;
use self::mutable_animation::*;
use self::core::*;
use self::flo_sqlite::*;

///
/// Database used to store an animation
/// 
pub struct AnimationDb {
    /// The core contains details of the database
    core: Arc<Desync<AnimationDbCore>>,

    /// The editor is used to provide the mutable animation interface (we keep it around so it can cache values if necessary)
    editor: Mutex<AnimationEditor>
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

        let db      = AnimationDb {
            core:   core,
            editor: Mutex::new(editor)
        };

        db
    }

    ///
    /// Creates an animation database that uses an existing database already set up in a SQLite connection
    /// 
    pub fn from_connection(connection: Connection) -> AnimationDb {
        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));
        let editor  = AnimationEditor::new(&core);

        let db = AnimationDb {
            core:   core,
            editor: Mutex::new(editor)
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
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore) -> Result<()>>(&self, action: TFn) {
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
}

impl AnimationDbCore {
    ///
    /// Creates a new database core
    /// 
    fn new(connection: Connection) -> AnimationDbCore {
        let core = AnimationDbCore {
            db:             FloSqlite::new(connection),
            failure:        None
        };

        core
    }

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
