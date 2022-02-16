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

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    ///
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        unimplemented!()
    }
}
