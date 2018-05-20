use super::*;

use animation::*;
use futures::*;

use std::ops::{Deref, Range};
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
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<'a+Deref<Target='a+Layer>>> {
        self.animation.get_layer_with_id(layer_id)
    }

    ///
    /// Retrieves the total number of items that have been performed on this animation
    /// 
    fn get_num_edits(&self) -> usize {
        self.animation.get_num_edits()
    }

    ///
    /// Reads from the edit log for this animation
    /// 
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> {
        self.animation.read_edit_log(range)
    }
}

///
/// Sink used to send data to the animation
/// 
struct FloModelSink<TargetSink, ProcessingFn> {
    /// Function called on every start send
    processing_fn: ProcessingFn,

    /// Sink where requests should be forwarded to 
    target_sink: TargetSink
}

impl<TargetSink, ProcessingFn> FloModelSink<TargetSink, ProcessingFn> {
    ///
    /// Creates a new model sink
    /// 
    pub fn new(target_sink: TargetSink, processing_fn: ProcessingFn) -> FloModelSink<TargetSink, ProcessingFn> {
        FloModelSink {
            processing_fn:  processing_fn,
            target_sink:    target_sink
        }
    }
}

impl<TargetSink: Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>, ProcessingFn: FnMut(&Vec<AnimationEdit>) -> ()> Sink for FloModelSink<TargetSink, ProcessingFn> {
    type SinkItem   = Vec<AnimationEdit>;
    type SinkError  = ();

    fn start_send(&mut self, item: Vec<AnimationEdit>) -> StartSend<Vec<AnimationEdit>, ()> {
        (self.processing_fn)(&item);

        self.target_sink.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        self.target_sink.poll_complete()
    }
}

impl<Anim: Animation+EditableAnimation> EditableAnimation for FloModel<Anim> {
    ///
    /// Retrieves a sink that can be used to send edits for this animation
    /// 
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    /// 
    fn edit<'a>(&'a self) -> Box<'a+Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>> {
        // Edit the underlying animation
        let animation_edit  = self.animation.edit();

        // Borrow the bits of the viewmodel we can change
        let frame_edit_counter  = &self.frame_edit_counter;
        let size_binding        = &self.size_binding;

        // Pipe the edits so they modify the model as a side-effect
        let model_edit          = FloModelSink::new(animation_edit, move |edits: &Vec<AnimationEdit>| {
            use self::AnimationEdit::*;
            use self::ElementEdit::*;
            use self::LayerEdit::*;

            // Update the viewmodel based on the edits that are about to go through
            let mut advance_edit_counter = false;

            for edit in edits.iter() {
                match edit {
                    SetSize(width, height) => {
                        size_binding.clone().set((*width, *height));
                        advance_edit_counter = true;
                    },

                    AddNewLayer(_)              |
                    RemoveLayer(_)              |
                    Element(_, _, Move(_, _))   |
                    Layer(_, Paint(_, _))       => {
                        advance_edit_counter = true;
                    }

                    Layer(_, AddKeyFrame(_))    |
                    Layer(_, RemoveKeyFrame(_)) => {
                        ()
                    },
                }
            }

            // Advancing the frame edit counter causes any animation frames to be regenerated
            if advance_edit_counter {
                frame_edit_counter.clone().set(self.frame_edit_counter.get()+1);
            }
        });

        Box::new(model_edit)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use animation::inmemory::*;
    use futures::executor;

    #[test]
    fn size_command_updates_size_binding() {
        let model = FloModel::new(InMemoryAnimation::new());

        // Initial size is 1980x1080
        assert!(model.size()        == (1980.0, 1080.0));
        assert!(model.size.get()    == (1980.0, 1080.0));

        // Change to 800x600
        {
            let mut edit_log = executor::spawn(model.edit());
            edit_log.wait_send(vec![AnimationEdit::SetSize(800.0, 600.0)]).unwrap();
        }

        // Binding should get changed by this edit
        assert!(model.size()        == (800.0, 600.0));
        assert!(model.size.get()    == (800.0, 600.0));
    }
}