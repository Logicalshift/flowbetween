use super::db_update::*;

// TODO: make error type more generic
use rusqlite::*;
use std::time::Duration;

///
/// Trait implemented by objects that can provide an underlying store for FlowBetween
/// 
pub trait FloStore {
    ///
    /// Performs a set of updates on the store
    /// 
    fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()>;

    ///
    /// Starts queuing up store updates for later execution as a batch
    /// 
    fn begin_queuing(&mut self);

    ///
    /// Executes the queued events (and stops queueing future events)
    /// 
    fn execute_queue(&mut self) -> Result<()>;

    ///
    /// Ensures any pending updates are committed to the database (but continues to queue future events)
    /// 
    fn flush_pending(&mut self) -> Result<()>;
}

///
/// Trait implemented by objects that can query an underlying store for FlowBetween
/// 
pub trait FloQuery {
    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<i64>;

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64) -> Result<Vec<Duration>>;

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64)>;

    ///
    /// Returns the assigned layer IDs
    /// 
    fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>>;
}

///
/// Trait implemented by objects that can read or write from a Flo data file
/// 
pub trait FloFile : FloStore + FloQuery {

}
