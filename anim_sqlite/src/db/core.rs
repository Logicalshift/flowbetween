use super::editlog::*;

use rusqlite::*;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore {
    /// The database connection
    pub sqlite: Connection,

    /// The enum values for the edit log (or None if these are not yet available)
    pub edit_log_enum: Option<EditLogEnumValues>,

    /// The ID value of the animation we're editing
    pub animation_id: i64,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,
}

impl AnimationDbCore {
    ///
    /// Performs an edit on this core if the failure condition is clear
    /// 
    pub fn edit<TEdit: Fn(&Connection, i64) -> Result<()>+Send>(&mut self, edit: TEdit) {
        // Perform the edit if there is no failure
        if self.failure.is_none() {
            self.failure = edit(&self.sqlite, self.animation_id).err();
        }
    }
}
