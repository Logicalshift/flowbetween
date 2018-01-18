use super::animation_database::*;

use rusqlite::*;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore {
    /// The database connection
    pub db: AnimationDatabase,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,
}

impl AnimationDbCore {
    ///
    /// Performs an edit on this core if the failure condition is clear
    /// 
    pub fn edit<TEdit: FnOnce(&mut AnimationDatabase) -> Result<()>>(&mut self, edit: TEdit) {
        // Perform the edit if there is no failure
        if self.failure.is_none() {
            self.failure = edit(&mut self.db).err();
        }
    }
}
