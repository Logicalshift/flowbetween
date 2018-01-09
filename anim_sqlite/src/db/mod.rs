use desync::*;
use rusqlite::*;
use std::mem;

#[cfg(test)] mod tests;
mod setup;

pub use self::setup::*;

///
/// Database used to store an animation
/// 
pub struct AnimationDb {
    core: Desync<AnimationDbCore>
}

///
/// Core data structure used by the animation database
/// 
struct AnimationDbCore {
    /// The database connection
    sqlite: Connection,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work if there has been an error
    failure: Option<Error>
}

impl AnimationDb {
    ///
    /// Creates a new animation database with an in-memory database
    /// 
    pub fn new() -> AnimationDb {
        Self::from_connection(Connection::open_in_memory().unwrap())
    }

    ///
    /// Creates a new animation database using the specified SQLite connection
    /// 
    pub fn new_from_connection(connection: Connection) -> AnimationDb {
        let db = Self::from_connection(connection);
        db.setup();
        db
    }

    ///
    /// Creates an animation database that uses an existing database already set up in a SQLite connection
    /// 
    pub fn from_connection(connection: Connection) -> AnimationDb {
        let core = AnimationDbCore {
            sqlite:     connection,
            failure:    None
        };

        AnimationDb {
            core: Desync::new(core)
        }
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
}
