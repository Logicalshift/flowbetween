use super::super::traits::*;

use std::mem;
use std::ops::Range;

///
/// An in-memory edit log implementation
/// 
pub struct InMemoryEditLog<Edit> {
    /// The edits stored in this log
    edits: Vec<Edit>
}

impl<Edit> InMemoryEditLog<Edit> {
    ///
    /// Creates a new in-memory edit log
    /// 
    pub fn new() -> InMemoryEditLog<Edit> {
        InMemoryEditLog {
            edits:      vec![],
        }
    }

    ///
    /// Commits some edits to this log
    /// 
    pub fn commit_edits<EditIterator: IntoIterator<Item=Edit>>(&mut self, new_edits: EditIterator) {
        self.edits.extend(new_edits.into_iter())
    }
}

impl<Edit: Clone> EditLog<Edit> for InMemoryEditLog<Edit> {
    fn length(&self) -> usize {
        self.edits.len()
    }

    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<Edit> {
        let len = self.edits.len();

        indices
            .filter(|index| *index < len)
            .map(|index| self.edits[index].clone())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initial_edit_length_is_zero() {
        let log = InMemoryEditLog::<i32>::new();
        assert!(log.length() == 0);
    }

    #[test]
    fn can_box_edit_log() {
        // (More of a 'this should be valid as an object' test)
        let log = Box::new(InMemoryEditLog::<i32>::new());
        assert!(log.length() == 0);
        assert!(log.read_iter(0..4).len() == 0);
    }

    #[test]
    fn can_commit_pending_and_read_them_back() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        assert!(log.length() == 0);

        let commit_range = log.commit_pending();
        assert!(commit_range == (0..4));
        assert!(log.length() == 4);

        log.set_pending(&[7,8,9,10]);
        let commit_range = log.commit_pending();
        assert!(commit_range == (4..8));
        assert!(log.length() == 8);

        assert!(log.read_iter(2..6) == vec![3, 4, 7, 8]);
    }

    #[test]
    fn can_cancel_pending() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        assert!(log.length() == 0);

        log.commit_pending();
        assert!(log.length() == 4);

        log.set_pending(&[7,8,9,10]);
        log.cancel_pending();
        assert!(log.length() == 4);

        assert!(log.read_iter(2..4) == vec![3, 4]);
    }

    #[test]
    fn committing_after_cancel_commits_nothing() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        assert!(log.length() == 0);

        log.commit_pending();
        assert!(log.length() == 4);

        log.set_pending(&[7,8,9,10]);
        log.cancel_pending();
        let commit_range = log.commit_pending();
        assert!(log.length() == 4);
        assert!(commit_range == (4..4));

        assert!(log.read_iter(2..4) == vec![3, 4]);
    }

    #[test]
    fn can_read_outside_of_bounds() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        log.commit_pending();

        assert!(log.read_iter(2..6) == vec![3, 4]);
        assert!(log.read_iter(100..300).len() == 0);
    }
}
