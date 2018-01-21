use super::*;

///
/// Provides the editlog trait for the animation DB
/// 
pub struct DbEditLog<TFile: FloFile+Send> {
    core: Arc<Desync<AnimationDbCore<TFile>>>
}

impl<TFile: FloFile+Send> DbEditLog<TFile> {
    ///
    /// Creates a new edit log for an animation database
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore<TFile>>>) -> DbEditLog<TFile> {
        DbEditLog {
            core: Arc::clone(core)
        }
    }
}

impl<TFile: FloFile+Send+'static> EditLog<AnimationEdit> for DbEditLog<TFile> {
    ///
    /// Retrieves the number of edits in this log
    ///
    fn length(&self) -> usize {
        self.core.sync(|core| {
            core.db.query_edit_log_length().unwrap() as usize
        })
    }

    ///
    /// Reads a range of edits from this log
    /// 
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<AnimationEdit> {
        unimplemented!()
    }
}
