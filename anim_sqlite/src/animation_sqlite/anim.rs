use super::*;

use animation::*;

use futures::*;
use rusqlite::*;
use std::time::Duration;
use std::ops::{Deref, Range};

impl SqliteAnimation {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    pub fn retrieve_and_clear_error(&self) -> Option<Error> {
        self.db.retrieve_and_clear_error()
    }

    ///
    /// Panics if this animation has reached an error condition
    ///
    pub fn panic_on_error(&self) {
        self.retrieve_and_clear_error().map(|erm| panic!("{:?}", erm));
    }
}

impl Animation for SqliteAnimation {
    #[inline]
    fn size(&self) -> (f64, f64) {
        self.db.size()
    }

    #[inline]
    fn get_layer_ids(&self) -> Vec<u64> {
        self.db.get_layer_ids()
    }

    fn duration(&self) -> Duration {
        self.db.duration()
    }

    fn frame_length(&self) -> Duration {
        self.db.frame_length()
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<'a+Deref<Target='a+Layer>>> {
        // Try to retrieve the layer from the editor
        let layer = self.db.get_layer_with_id(layer_id);

        // Turn into a reader if it exists
        let layer = layer.map(|layer| {
            let boxed: Box<Layer> = Box::new(layer);
            boxed
        });

        layer.map(|layer| {
            let boxed: Box<'a+Deref<Target='a+Layer>> = Box::new(layer);
            boxed
        })
    }

    fn get_num_edits(&self) -> usize {
        self.db.get_num_edits().unwrap_or(0)
    }

    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> {
        self.db.read_edit_log(range)
    }

    fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>> {
        unimplemented!()
    }

    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        unimplemented!()
    }
}
