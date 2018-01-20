use std::time::Duration;

use rusqlite::*;

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
