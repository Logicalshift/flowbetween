use crate::traits::*;

use futures::stream::{BoxStream};

use ::desync::*;
use flo_stream::*;

use std::sync::*;
use std::time::{Duration};
use std::ops::{Range};

///
/// Adds undo support to another animation type
///
pub struct UndoableAnimation<Anim: 'static+Unpin+EditableAnimation> {
    /// The animation that this will add undo support to
    animation:  Arc<Desync<Anim>>,

    /// The input stream of edits for this animation
    edits:      Publisher<Arc<Vec<AnimationEdit>>>,
}

impl<Anim: Unpin+EditableAnimation> UndoableAnimation<Anim> {
    ///
    /// Adds undo support to an existing animation
    ///
    pub fn new(animation: Anim) -> UndoableAnimation<Anim> {
        // Box up the animation and create the edit stream
        let animation   = Arc::new(Desync::new(animation));
        let edits       = Publisher::new(10);

        UndoableAnimation {
            animation,
            edits,
        }
    }
}

impl<Anim: 'static+Unpin+EditableAnimation> Animation for UndoableAnimation<Anim> {
    ///
    /// Retrieves the frame size of this animation
    ///
    fn size(&self) -> (f64, f64) { 
        self.animation.sync(|anim| anim.size()) 
    }

    ///
    /// Retrieves the length of this animation
    ///
    fn duration(&self) -> Duration { 
        self.animation.sync(|anim| anim.duration()) 
    }

    ///
    /// Retrieves the duration of a single frame
    ///
    fn frame_length(&self) -> Duration { 
        self.animation.sync(|anim| anim.frame_length()) 
    }

    ///
    /// Retrieves the IDs of the layers in this object
    ///
    fn get_layer_ids(&self) -> Vec<u64> { 
        self.animation.sync(|anim| anim.get_layer_ids()) 
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    ///
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<dyn Layer>> { 
        self.animation.sync(|anim| anim.get_layer_with_id(layer_id)) 
    }

    ///
    /// Retrieves the total number of items that have been performed on this animation
    ///
    fn get_num_edits(&self) -> usize { 
        self.animation.sync(|anim| anim.get_num_edits()) 
    }

    ///
    /// Reads from the edit log for this animation
    ///
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> BoxStream<'a, AnimationEdit> {
        unimplemented!()
        // self.animation.sync(|anim| anim.read_edit_log(range))
    }
}

impl<Anim: 'static+Unpin+EditableAnimation> EditableAnimation for UndoableAnimation<Anim> {
    ///
    /// Assigns a new unique ID for creating a new motion
    ///
    /// This ID will not have been used so far and will not be used again, and can be used as the ID for the MotionElement vector element.
    ///
    fn assign_element_id(&self) -> ElementId {
        self.animation.sync(|anim| anim.assign_element_id())
    }

    ///
    /// Retrieves a sink that can be used to send edits for this animation
    ///
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    ///
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.edits.republish()
    }

    ///
    /// Sends a set of edits straight to this animation
    /// 
    /// (Note that these are not always published to the publisher)
    ///
    fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        // TODO
        unimplemented!()
    }

    ///
    /// Returns a stream of edits as they are being retired (ie, the edits that are now visible on the animation)
    ///
    fn retired_edits(&self) -> BoxStream<'static, Arc<Vec<AnimationEdit>>> {
        self.animation.sync(|anim| anim.retired_edits())
    }

    ///
    /// Flushes any caches this might have (forces reload from data storage)
    ///
    fn flush_caches(&self) {
        self.animation.sync(|anim| anim.flush_caches())
    }
}
