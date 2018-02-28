use super::*;

use animation::*;

use std::time::Duration;

impl<Anim: Animation> Animation for FloModel<Anim> {
    ///
    /// Retrieves the frame size of this animation
    /// 
    fn size(&self) -> (f64, f64) {
        self.animation.size()
    }

    ///
    /// Retrieves the length of this animation
    /// 
    fn duration(&self) -> Duration {
        self.animation.duration()
    }

    ///
    /// Retrieves the duration of a single frame
    /// 
    fn frame_length(&self) -> Duration {
        self.animation.frame_length()
    }

    ///
    /// Retrieves the IDs of the layers in this object
    /// 
    fn get_layer_ids(&self) -> Vec<u64> {
        self.animation.get_layer_ids()
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    /// 
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Reader<'a, Layer>> {
        self.animation.get_layer_with_id(layer_id)
    }

    ///
    /// Retrieves the log for this animation
    /// 
    fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>> {
        self.animation.get_log()
    }

    ///
    /// Retrieves an edit log that can be used to alter this animation
    /// 
    fn edit<'a>(&'a self) -> Editor<'a, PendingEditLog<AnimationEdit>> {
        // TODO: want to return an edit log that updates this object as well
        // TODO: make sure the updates are properly serialised when they come from multiple sources
        self.animation.edit()
    }

    ///
    /// Retrieves an edit log that can be used to edit a layer in this animation
    /// 
    fn edit_layer<'a>(&'a self, layer_id: u64) -> Editor<'a, PendingEditLog<LayerEdit>> {
        // TODO: need the edit log here as well
        self.animation.edit_layer(layer_id)
    }
}
