use super::db_update::*;
use super::flo_query::*;

// TODO: make error type more generic
use rusqlite::*;

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
/// Trait implemented by objects that can read or write from a Flo data file
/// 
pub trait FloFile : FloStore + FloQuery {

}
