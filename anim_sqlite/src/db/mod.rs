use animation::*;

use desync::*;
use rusqlite::*;

use std::mem;
use std::sync::*;
use std::time::Duration;

#[cfg(test)] mod tests;

mod animation_database;
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
use self::animation_database::*;

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
        AnimationDatabase::setup(&connection);

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
            let mut failure = None;
            mem::swap(&mut core.failure, &mut failure);

            failure
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
            db:             AnimationDatabase::new(connection),
            vector_enum:    None,
            failure:        None
        };

        core
    }

    ///
    /// Turns a microsecond count into a duration
    /// 
    fn from_micros(when: i64) -> Duration {
        Duration::new((when / 1_000_000) as u64, ((when % 1_000_000) * 1000) as u32)
    }

    ///
    /// Retrieves microseconds from a duration
    /// 
    fn get_micros(when: &Duration) -> i64 {
        let secs:i64    = when.as_secs() as i64;
        let nanos:i64   = when.subsec_nanos() as i64;

        (secs * 1_000_000) + (nanos / 1_000)
    }
}
