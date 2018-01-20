use super::*;
use super::flo_query::*;

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
    /// Queries the active layer IDs for the animation
    /// 
    pub fn get_layer_ids(&self) -> Vec<u64> {
        self.core.sync(|core| {
            core.db.query_assigned_layer_ids()
        }).unwrap()
    }
}
