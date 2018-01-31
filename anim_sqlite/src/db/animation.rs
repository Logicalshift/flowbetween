use super::*;
use super::editlog::*;
use super::flo_query::*;

use std::time::Duration;

impl AnimationDb {
    ///
    /// Queries the size of the animation that this will edit
    /// 
    pub fn size(&self) -> (f64, f64) {
        self.core.sync(|core| {
            core.db.query_size()
        }).unwrap()
    }

    ///
    /// Queries the duration of this animation
    /// 
    pub fn duration(&self) -> Duration {
        self.core.sync(|core| {
            core.db.query_duration()
        }).unwrap()
    }

    ///
    /// Queries the frame length of this animation
    /// 
    pub fn frame_length(&self) -> Duration {
        self.core.sync(|core| {
            core.db.query_frame_length()
        }).unwrap()
    }

    ///
    /// Queries the active layer IDs for the animation
    /// 
    pub fn get_layer_ids(&self) -> Vec<u64> {
        self.core.sync(|core| {
            core.db.query_assigned_layer_ids()
        }).unwrap()
    }

    ///
    /// Retrieves an edit log reader for this animation
    /// 
    pub fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>> {
        // Create an edit log (and box it so we can return it in a reader)
        let edit_log                                    = DbEditLog::new(&self.core);
        let edit_log: Box<'a+EditLog<AnimationEdit>>    = Box::new(edit_log);

        Reader::new(edit_log)
    }
}
