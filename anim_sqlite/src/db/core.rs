use super::flo_store::*;

use rusqlite::*;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore<TFile: FloFile> {
    /// The database connection
    pub db: TFile,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,
}

impl<TFile: FloFile> AnimationDbCore<TFile> {
    ///
    /// Performs an edit on this core if the failure condition is clear
    /// 
    pub fn edit<TEdit: FnOnce(&mut TFile) -> Result<()>>(&mut self, edit: TEdit) {
        // Perform the edit if there is no failure
        if self.failure.is_none() {
            self.failure = edit(&mut self.db).err();
        }
    }
}
