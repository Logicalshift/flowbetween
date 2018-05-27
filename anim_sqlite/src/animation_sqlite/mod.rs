use super::db::*;

#[cfg(test)] mod tests;

mod data;
mod anim;
mod edit;

pub use self::data::*;
pub use self::anim::*;
pub use self::edit::*;

///
/// Animation that uses a SQLite database as a backing store
/// 
pub struct SqliteAnimation {
    /// The database for this animation
    db: AnimationDb
}
