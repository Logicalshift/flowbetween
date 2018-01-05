use super::super::traits::*;

use std::mem;

///
/// An in-memory edit log implementation
/// 
pub struct InMemoryEditLog<Edit> {
    /// The edits stored in this log
    edits: Vec<Edit>,

    /// The non-committed edits
    pending: Vec<Edit>
}

impl<Edit> InMemoryEditLog<Edit> {
    pub fn new() -> InMemoryEditLog<Edit> {
        InMemoryEditLog {
            edits:      vec![],
            pending:    vec![]
        }
    }
}

impl<Edit: Clone> EditLog<Edit> for InMemoryEditLog<Edit> {
    fn length(&self) -> usize {
        self.edits.len()
    }

    fn read<'a>(&'a self, start: usize, end: usize) -> Vec<&'a Edit> {
        let start   = if start > self.edits.len() { self.edits.len() } else { start };
        let end     = if end > self.edits.len() { self.edits.len() } else { end };

        self.edits[start..end].iter().collect()
    }

    fn pending(&self) -> Vec<Edit> {
        self.pending.clone()
    }

    fn set_pending(&mut self, edits: &[Edit]) {
        self.pending = edits.iter()
            .map(|edit| edit.clone())
            .collect();
    }

    fn commit_pending(&mut self) {
        // Get the list of pending edits
        let mut pending = vec![];
        mem::swap(&mut pending, &mut self.pending);

        // Move them into the edit list
        self.edits.extend(pending);
    }

    fn cancel_pending(&mut self) {
        self.pending = vec![];
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
    }

    #[test]
    fn can_commit_pending_and_read_them_back() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        assert!(log.length() == 0);

        log.commit_pending();
        assert!(log.length() == 4);

        log.set_pending(&[7,8,9,10]);
        log.commit_pending();
        assert!(log.length() == 8);

        assert!(log.read(2, 6) == vec![&3, &4, &7, &8]);
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

        assert!(log.read(2, 4) == vec![&3, &4]);
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
        log.commit_pending();
        assert!(log.length() == 4);

        assert!(log.read(2, 4) == vec![&3, &4]);
    }

    #[test]
    fn can_read_outside_of_bounds() {
        let mut log = InMemoryEditLog::<i32>::new();

        log.set_pending(&[1, 2, 3, 4]);
        log.commit_pending();

        assert!(log.read(2, 6) == vec![&3, &4]);
        assert!(log.read(100, 300).len() == 0);
    }
}
