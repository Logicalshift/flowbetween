use std::sync::*;
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
}

///
/// Trait implemented by edit logs representing a set of edits
/// waiting to be committed to another edit log.
/// 
pub trait PendingEditLog<Edit> {
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

///
/// Utility trait that provides some extra sugar for edit logs
/// 
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

//
// Convenience implementations that let us treat references to edit logs as
// edit logs themselves if we want. (Very useful when mapping or slicing
// existing logs)
//

impl<'a, Edit, Log: EditLog<Edit>> EditLog<Edit> for &'a mut Log {
    #[inline]
    fn length(&self) -> usize { 
        (**self).length()
    }

    #[inline]
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        (**self).read(indices)
    }
}

impl<'a, Edit, Log: PendingEditLog<Edit>> PendingEditLog<Edit> for &'a mut Log {
    #[inline]
    fn pending(&self) -> Vec<Edit> {
        (**self).pending()
    }

    #[inline]
    fn set_pending(&mut self, edits: &[Edit]) {
        (**self).set_pending(edits)
    }

    #[inline]
    fn commit_pending(&mut self) -> Range<usize> {
        (**self).commit_pending()
    }

    #[inline]
    fn cancel_pending(&mut self) {
        (**self).cancel_pending()
    }
}

impl<'a, Edit, Log: EditLog<Edit>> EditLog<Edit> for RwLockReadGuard<'a, Log> {
    #[inline]
    fn length(&self) -> usize { 
        (**self).length()
    }

    #[inline]
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        (**self).read(indices)
    }
}

impl<'a, Edit, Log: EditLog<Edit>> EditLog<Edit> for RwLockWriteGuard<'a, Log> {
    #[inline]
    fn length(&self) -> usize { 
        (**self).length()
    }

    #[inline]
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        (**self).read(indices)
    }
}

impl<'a, Edit, Log: PendingEditLog<Edit>> PendingEditLog<Edit> for RwLockWriteGuard<'a, Log> {
    #[inline]
    fn pending(&self) -> Vec<Edit> {
        (**self).pending()
    }

    #[inline]
    fn set_pending(&mut self, edits: &[Edit]) {
        (**self).set_pending(edits)
    }

    #[inline]
    fn commit_pending(&mut self) -> Range<usize> {
        (**self).commit_pending()
    }

    #[inline]
    fn cancel_pending(&mut self) {
        (**self).cancel_pending()
    }
}

impl<'a, Edit, Log: EditLog<Edit>> EditLog<Edit> for RwLock<Log> {
    #[inline]
    fn length(&self) -> usize { 
        self.read().unwrap().length()
    }

    #[inline]
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        self.read().unwrap().read(indices)
    }
}

impl<'a, Edit, Log: PendingEditLog<Edit>> PendingEditLog<Edit> for RwLock<Log> {
    #[inline]
    fn pending(&self) -> Vec<Edit> {
        self.read().unwrap().pending()
    }

    #[inline]
    fn set_pending(&mut self, edits: &[Edit]) {
        // TODO: race condition if multiple threads are calling set/commit pending (as we release the write lock)
        // Maybe want to fix so that threads each have their own 'pending' list or something
        // Using the write guard implementation also fixes this issue
        self.write().unwrap().set_pending(edits)
    }

    #[inline]
    fn commit_pending(&mut self) -> Range<usize> {
        self.write().unwrap().commit_pending()
    }

    #[inline]
    fn cancel_pending(&mut self) {
        self.write().unwrap().cancel_pending()
    }
}

impl<'a, Edit, Log: EditLog<Edit>> EditLog<Edit> for Arc<RwLock<Log>> {
    #[inline]
    fn length(&self) -> usize { 
        (**self).read().unwrap().length()
    }

    #[inline]
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        (**self).read().unwrap().read(indices)
    }
}

impl<'a, Edit, Log: PendingEditLog<Edit>> PendingEditLog<Edit> for Arc<RwLock<Log>> {
    #[inline]
    fn pending(&self) -> Vec<Edit> {
        (**self).read().unwrap().pending()
    }

    #[inline]
    fn set_pending(&mut self, edits: &[Edit]) {
        // TODO: race condition if multiple threads are calling set/commit pending (as we release the write lock)
        // Maybe want to fix so that threads each have their own 'pending' list or something
        // Using the write guard implementation also fixes this issue
        (**self).write().unwrap().set_pending(edits)
    }

    #[inline]
    fn commit_pending(&mut self) -> Range<usize> {
        (**self).write().unwrap().commit_pending()
    }

    #[inline]
    fn cancel_pending(&mut self) {
        (**self).write().unwrap().cancel_pending()
    }
}
