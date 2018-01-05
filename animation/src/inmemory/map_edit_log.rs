use super::super::traits::*;

use std::ops::Range;
use std::marker::PhantomData;

///
/// Edit log that maps a range of edits from a source edit type to a target
/// 
pub struct MapEditLog<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog>
where   TheirsToOurs: Fn(&TheirEdit) -> OurEdit,
        OursToTheirs: Fn(&OurEdit) -> TheirEdit
{
    source_log:     SourceLog,
    theirs_to_ours: TheirsToOurs,
    ours_to_theirs: OursToTheirs,

    our_phantom:    PhantomData<OurEdit>,
    their_phantom:  PhantomData<TheirEdit>
}

impl<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog> MapEditLog<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog>
where   TheirsToOurs: Fn(&TheirEdit) -> OurEdit,
        OursToTheirs: Fn(&OurEdit) -> TheirEdit {
    ///
    /// Creates a new edit log that maps edits
    /// 
    pub fn new(source_log: SourceLog, theirs_to_ours: TheirsToOurs, ours_to_theirs: OursToTheirs) -> MapEditLog<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog> {
        MapEditLog {
            source_log:     source_log,
            theirs_to_ours: theirs_to_ours,
            ours_to_theirs: ours_to_theirs,

            our_phantom:    PhantomData,
            their_phantom:  PhantomData
        }
    }
}

impl<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog: EditLog<TheirEdit>> EditLog<OurEdit> for MapEditLog<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog>
where   TheirsToOurs: Fn(&TheirEdit) -> OurEdit,
        OursToTheirs: Fn(&OurEdit) -> TheirEdit {
    fn length(&self) -> usize {
        self.source_log.length()
    }

    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<OurEdit> {
        self.source_log
            .read(indices)
            .into_iter()
            .map(|theirs| (self.theirs_to_ours)(&theirs))
            .collect()
    }

    fn pending(&self) -> Vec<OurEdit> {
        self.source_log
            .pending()
            .into_iter()
            .map(|theirs| (self.theirs_to_ours)(&theirs))
            .collect()
    }
}

impl<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog: MutableEditLog<TheirEdit>> MutableEditLog<OurEdit> for MapEditLog<OurEdit, TheirEdit, TheirsToOurs, OursToTheirs, SourceLog>
where   TheirsToOurs: Fn(&TheirEdit) -> OurEdit,
        OursToTheirs: Fn(&OurEdit) -> TheirEdit {
    fn set_pending(&mut self, edits: &[OurEdit]) {
        let their_pending: Vec<TheirEdit> = edits.iter()
            .map(|ours| (self.ours_to_theirs)(ours))
            .collect();
        
        self.source_log.set_pending(&their_pending);
    }

    fn commit_pending(&mut self) -> Range<usize> {
        self.source_log.commit_pending()
    }

    fn cancel_pending(&mut self) {
        self.source_log.cancel_pending()
    }
}

#[cfg(test)]
mod test {
    use super::super::edit_log::*;
    use super::*;

    use std::sync::*;

    #[test]
    fn map_from_int_edit_log() {
        let mut integers = InMemoryEditLog::new();
        integers.set_pending(&[1, 2, 3, 4]);
        integers.commit_pending();

        let add_one = MapEditLog::new(integers, |i| i+1, |i| i-1);
        assert!(add_one.read_iter(0..4) == vec![2,3,4,5]);
    }

    #[test]
    fn map_updates_on_write() {
        let mut integers = Arc::new(RwLock::new(InMemoryEditLog::new()));
        integers.set_pending(&[1, 2, 3, 4]);
        integers.commit_pending();

        // Note race condition when using a RwLock log this way
        // Could do integers.write() here to avoid the issue
        let mut add_one = MapEditLog::new(integers.clone(), |i| i+1, |i| i-1);
        add_one.set_pending(&[5, 6, 7, 8]);
        add_one.commit_pending();

        assert!(add_one.read_iter(0..8) == vec![2, 3, 4, 5, 5, 6, 7, 8]);
        assert!(integers.read_iter(0..8) == vec![1, 2, 3, 4, 4, 5, 6, 7]);
    }
}
