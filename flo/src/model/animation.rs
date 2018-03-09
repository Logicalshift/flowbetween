use super::*;

use animation::*;
use animation::inmemory::pending_log::*;

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
        // TODO: make sure the updates are properly serialised when they come from multiple sources

        let mut animation_edit  = self.animation.edit();
        let model_edit          = InMemoryPendingLog::new(move |edits| {
            use self::AnimationEdit::*;
            use self::LayerEdit::*;

            // Post to the underlying animation
            animation_edit.set_pending(&edits);
            animation_edit.commit_pending();

            // Update the viewmodel based on the edits
            let mut advance_edit_counter = false;

            for edit in edits {
                match edit {
                    SetSize(width, height) => {
                        self.size_binding.clone().set((width, height));
                        advance_edit_counter = true;
                    },

                    AddNewLayer(_)          |
                    RemoveLayer(_)          |
                    Layer(_, Paint(_, _))   => {
                        advance_edit_counter = true;
                    }

                    Layer(_, AddKeyFrame(_))    |
                    Layer(_, RemoveKeyFrame(_)) => {
                        ()
                    }
                }
            }

            // Advancing the frame edit counter causes any animation frames to be regenerated
            if advance_edit_counter {
                self.frame_edit_counter.clone().set(self.frame_edit_counter.get()+1)
            }
        });

        let edit_log: Box<'a+PendingEditLog<AnimationEdit>> = Box::new(model_edit);
        Editor::new(edit_log)
    }

    ///
    /// Retrieves an edit log that can be used to edit a layer in this animation
    /// 
    fn edit_layer<'a>(&'a self, layer_id: u64) -> Editor<'a, PendingEditLog<LayerEdit>> {
        // TODO: need the edit log here as well
        self.animation.edit_layer(layer_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use animation::inmemory::*;

    #[test]
    fn size_command_updates_size_binding() {
        let model = FloModel::new(InMemoryAnimation::new());

        // Initial size is 1980x1080
        assert!(model.size()        == (1980.0, 1080.0));
        assert!(model.size.get()    == (1980.0, 1080.0));

        // Change to 800x600
        {
            let mut edit_log = model.edit();
            edit_log.set_pending(&vec![AnimationEdit::SetSize(800.0, 600.0)]);
            edit_log.commit_pending();
        }

        // Binding should get changed by this edit
        assert!(model.size()        == (800.0, 600.0));
        assert!(model.size.get()    == (800.0, 600.0));
    }
}