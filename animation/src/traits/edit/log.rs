use std::ops::Range;

///
/// Trait implemented by things that can be edited (or described) by sequences
/// of commands
/// 
pub trait EditLog<Edit> {
    ///
    /// Retrieves the number of edits in this log
    ///
    fn length(&self) -> usize;

    ///
    /// Reads a range of edits from this log
    /// 
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit>;

    ///
    /// The current set of pending edits
    /// 
    fn pending(&self) -> Vec<Edit>;

    ///
    /// Sets the pending edits for this log (replacing any existing
    /// pending edits)
    /// 
    fn set_pending(&mut self, edits: &[Edit]);

    ///
    /// Commits any pending edits for this log. Returns the
    /// range where the edits were committed.
    /// 
    fn commit_pending(&mut self) -> Range<usize>;

    ///
    /// Cancels any pending edits for this log
    /// 
    fn cancel_pending(&mut self);

    // TODO: undos, redos?
}

pub trait EditLogUtils<Edit> {
    ///
    /// Convenience version of read that works on an IntoIterator type
    /// 
    fn read_iter<ToIterator: IntoIterator<Item=usize>>(&self, items: ToIterator) -> Vec<Edit>;
}

impl<TEditLog: EditLog<Edit>, Edit> EditLogUtils<Edit> for TEditLog {
    #[inline]
    fn read_iter<ToIterator: IntoIterator<Item=usize>>(&self, items: ToIterator) -> Vec<Edit> {
        self.read(&mut items.into_iter())
    }
}
