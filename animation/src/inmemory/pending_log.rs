use super::super::traits::*;

use std::mem;
use std::ops::Range;

///
/// Represents an in-memory pending edit log
/// 
pub struct InMemoryPendingLog<Edit, CommitFn> {
    /// The current set of pending edits
    pending: Vec<Edit>,

    /// Function used to commit data to this log
    on_commit: CommitFn
}

impl<Edit, CommitFn> InMemoryPendingLog<Edit, CommitFn>
where CommitFn: FnMut(Vec<Edit>) -> Range<usize> {
    ///
    /// Creates a new in-memory pending log
    /// 
    pub fn new(on_commit: CommitFn) -> InMemoryPendingLog<Edit, CommitFn> {
        InMemoryPendingLog {
            pending:    vec![],
            on_commit:  on_commit
        }
    }
}

impl<Edit: Clone, CommitFn> PendingEditLog<Edit> for InMemoryPendingLog<Edit, CommitFn>
where CommitFn: FnMut(Vec<Edit>) -> Range<usize> {
    fn pending(&self) -> Vec<Edit> {
        self.pending.clone()
    }

    fn set_pending(&mut self, edits: &[Edit]) {
        self.pending = edits.iter().map(|edit| edit.clone()).collect();
    }

    fn commit_pending(&mut self) -> Range<usize> {
        // Swap out the pending edits for an empty lsit
        let mut pending = vec![];
        mem::swap(&mut pending, &mut self.pending);

        // Pass to the on_commit function
        (self.on_commit)(pending)
    }

    fn cancel_pending(&mut self) {
        self.pending = vec![];
    }
}
