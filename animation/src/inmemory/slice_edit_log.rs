use super::super::traits::*;

use std::ops::Range;
use std::marker::PhantomData;

///
/// Represents an edit log that corresponds to a slice from another edit log
/// 
pub struct SliceEditLog<Edit, SourceLog> {
    /// The log that is the source of this slice
    source_log: SourceLog,

    /// The indexes of the edits that are included in this log
    included: Vec<usize>,

    phantom_edit: PhantomData<Edit>
}

impl<Edit, SourceLog> SliceEditLog<Edit, SourceLog> {
    ///
    /// Creates an edit log representing a slice of another edit log
    /// 
    pub fn new<IncludedIter: IntoIterator<Item=usize>>(source_log: SourceLog, items: IncludedIter) -> SliceEditLog<Edit, SourceLog> {
        SliceEditLog {
            source_log:     source_log,
            included:       items.into_iter().collect(),
            phantom_edit:   PhantomData
        }        
    }
}

impl<Edit, SourceLog: EditLog<Edit>> EditLog<Edit> for SliceEditLog<Edit, SourceLog> {
    fn length(&self) -> usize {
        self.included.len()
    }

    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        let len                 = self.included.len();
        let mut source_indices  = indices
            .filter(|index| *index < len)
            .map(|index| self.included[index]);
        
        self.source_log.read(&mut source_indices)
    }

    fn pending(&self) -> Vec<Edit> {
        self.source_log.pending()
    }
}

impl<Edit, SourceLog: MutableEditLog<Edit>> MutableEditLog<Edit> for SliceEditLog<Edit, SourceLog> {
    fn set_pending(&mut self, edits: &[Edit]) {
        self.source_log.set_pending(edits)
    }

    fn commit_pending(&mut self) -> Range<usize> {
        // Committing pending items here also adds them to this log (committing elsewhere does not!)
        let committed = self.source_log.commit_pending();

        // Add the committed items to the included list
        let start_pos = self.included.len();
        self.included.extend(committed.into_iter());
        let end_pos = self.included.len();

        // Result is the range in our list
        start_pos..end_pos
    }

    fn cancel_pending(&mut self) {
        self.source_log.cancel_pending()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::edit_log::*;

    #[test]
    fn slice_existing_log() {
        let mut integers = InMemoryEditLog::new();
        integers.set_pending(&[1,2,3,4,5,6,7,8]);
        integers.commit_pending();

        let slice = SliceEditLog::new(&mut integers, vec![2,4,6,8]);

        assert!(slice.length() == 4);
        assert!(slice.read_iter(1..3) == vec![5,7]);
    }

    #[test]
    fn commit_to_existing_sliced_log() {
        let mut integers = InMemoryEditLog::new();
        integers.set_pending(&[1,2,3,4,5,6,7,8]);
        integers.commit_pending();

        let mut slice = SliceEditLog::new(&mut integers, vec![2,4,6,8]);

        slice.set_pending(&[9, 10, 11]);
        let committed_range = slice.commit_pending();

        assert!(committed_range == (4..7));
        assert!(slice.read_iter(4..7) == vec![9, 10, 11]);
    }
}
