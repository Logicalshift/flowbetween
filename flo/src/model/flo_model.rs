use super::tools::*;
use super::frame::*;
use super::timeline::*;
use super::selection::*;
use super::onion_skin::*;

use flo_stream::*;
use flo_binding::*;
use flo_animation::*;
use futures::*;
use futures::stream::{BoxStream};
use ::desync::*;

use std::ops::Range;
use std::time::Duration;
use std::sync::*;

///
/// The model for the animation editor
///
pub struct FloModel<Anim: Animation> {
    /// The animation that is being edited
    animation: Arc<Anim>,

    /// The status of the currently selected tool
    tools: ToolModel<Anim>,

    /// The timeline model
    timeline: TimelineModel<Anim>,

    /// The frame model
    frame: FrameModel,

    /// The selection model
    selection: SelectionModel,

    /// The onion skin model
    onion_skin: OnionSkinModel<Anim>,

    /// The size of the animation
    pub size: BindRef<(f64, f64)>,

    /// The underlying size binding
    size_binding: Binding<(f64, f64)>,

    /// Counter used to set an edit ID for the frame (essentially indicates when the frame has been redrawn)
    frame_edit_counter: Binding<u64>,

    /// Publisher where we send edits to this stream
    edit_publisher: Arc<Desync<Publisher<Arc<Vec<AnimationEdit>>>>>
}

impl<Anim: EditableAnimation+Animation+'static> FloModel<Anim> {
    ///
    /// Creates a new model
    ///
    pub fn new(animation: Anim) -> FloModel<Anim> {
        let mut edit_publisher  = animation.edit();
        let animation           = Arc::new(animation);
        let tools               = ToolModel::new();
        let timeline            = TimelineModel::new(Arc::clone(&animation), edit_publisher.subscribe());
        let frame_edit_counter  = bind(0);
        let frame               = FrameModel::new(Arc::clone(&animation), edit_publisher.subscribe(), BindRef::new(&timeline.current_time), BindRef::new(&frame_edit_counter), BindRef::new(&timeline.selected_layer));
        let selection           = SelectionModel::new(&frame, &timeline);
        let onion_skin          = OnionSkinModel::new(Arc::clone(&animation), &timeline);

        let size_binding        = bind(animation.size());
        let edit_publisher      = Arc::new(Desync::new(edit_publisher));

        let mut model           = FloModel {
            animation:          animation,
            tools:              tools,
            timeline:           timeline,
            frame_edit_counter: frame_edit_counter,
            frame:              frame,
            selection:          selection,
            onion_skin:         onion_skin,

            size:               BindRef::from(size_binding.clone()),
            size_binding:       size_binding,

            edit_publisher:     edit_publisher
        };

        model.subscribe_to_animation_edits();

        model
    }

    ///
    /// Updates the model based on edits made to the animation
    ///
    fn subscribe_to_animation_edits(&mut self) {
        // Subscribe to the edits posted to the model and gather them together
        let subscription            = self.edit_publisher.sync(|publisher| publisher.subscribe());

        // Gather together the properties we're going to update
        let size_binding            = self.size_binding.clone();
        let timeline                = self.timeline.clone();
        let frame_edit_counter      = self.frame_edit_counter.clone();

        // Process edits for this subscription
        pipe_in(Arc::clone(&self.edit_publisher), subscription, move |_, edits| {
            Self::process_edits(&*edits, &size_binding, &timeline, &frame_edit_counter);
            future::ready(()).boxed()
        });
    }

    ///
    /// Updates the model based on edits to the animation
    ///
    fn process_edits(edits: &Vec<AnimationEdit>, size_binding: &Binding<(f64, f64)>, timeline: &TimelineModel<Anim>, frame_edit_counter: &Binding<u64>) {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        // Update the viewmodel based on the edits that are about to go through
        let mut advance_edit_counter = false;

        for edit in edits.iter() {
            match edit {
                SetSize(width, height) => {
                    size_binding.set((*width, *height));
                    advance_edit_counter = true;
                },

                AddNewLayer(_)              |
                RemoveLayer(_)              |
                Element(_, _)               |
                Motion(_, _)                |
                Layer(_, Path(_, _))        |
                Layer(_, Paint(_, _))       => {
                    advance_edit_counter = true;
                }

                Layer(_, AddKeyFrame(_))    |
                Layer(_, RemoveKeyFrame(_)) => {
                    advance_edit_counter = true;
                },

                Layer(layer_id, SetName(new_name)) => {
                    timeline.layers.get()
                        .iter()
                        .for_each(|layer| if &layer.id == layer_id { layer.name.set(new_name.clone())} );
                    advance_edit_counter = true;
                },

                Layer(layer_id, SetOrdering(at_index)) => {
                    unimplemented!("Cannot update model with layer ordering yet")
                }
            }
        }

        // Advancing the frame edit counter causes any animation frames to be regenerated
        if advance_edit_counter {
            frame_edit_counter.set(frame_edit_counter.get()+1);
        }
    }

    ///
    /// Returns a future that indicates when all of the pending edits have been processed
    ///
    pub fn when_complete(&self) -> impl Future<Output=()> {
        let edit_publisher  = self.edit_publisher.clone();
        let when_empty      = self.animation.edit().when_empty();

        async move {
            // Wait for all of the pending edits to be published
            when_empty.await;

            // Wait for the edit publisher to finish processing them
            edit_publisher.future(|_| Box::pin(future::ready(()))).await.ok();
        }
    }
}

impl<Anim: Animation+'static> FloModel<Anim> {
    ///
    /// Retrieves the model for the drawing tools for this animation
    ///
    pub fn tools(&self) -> &ToolModel<Anim> {
        &self.tools
    }

    ///
    /// Retrieves the model of the timeline for this animation
    ///
    pub fn timeline(&self) -> &TimelineModel<Anim> {
        &self.timeline
    }

    ///
    /// Retrieves the frame model for this animation
    ///
    pub fn frame(&self) -> &FrameModel {
        &self.frame
    }

    ///
    /// Retrieves the selection model for this animation
    ///
    pub fn selection(&self) -> &SelectionModel {
        &self.selection
    }

    ///
    /// Retrieves the onion skin model for this animation
    ///
    pub fn onion_skin(&self) -> &OnionSkinModel<Anim> {
        &self.onion_skin
    }

    ///
    /// Retrieves the frame update binding for this animation
    ///
    pub fn frame_update_count(&self) -> BindRef<u64> {
        BindRef::from(self.frame_edit_counter.clone())
    }

    ///
    /// Returns a stream containing any edits that have occurred on this stream
    ///
    pub fn subscribe_edits(&self) -> impl Stream<Item=Arc<Vec<AnimationEdit>>>+Unpin+Clone+Send {
        self.edit_publisher.sync(|publisher| publisher.subscribe())
    }
}

// Clone because for some reason #[derive(Clone)] does something weird
impl<Anim: Animation> Clone for FloModel<Anim> {
    fn clone(&self) -> FloModel<Anim> {
        FloModel {
            animation:          self.animation.clone(),
            tools:              self.tools.clone(),
            timeline:           self.timeline.clone(),
            frame_edit_counter: self.frame_edit_counter.clone(),
            frame:              self.frame.clone(),
            selection:          self.selection.clone(),
            onion_skin:         self.onion_skin.clone(),

            size:               self.size.clone(),
            size_binding:       self.size_binding.clone(),

            edit_publisher:     self.edit_publisher.clone()
        }
    }
}

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
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Arc<dyn Layer>> {
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
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> BoxStream<'a, AnimationEdit> {
        self.animation.read_edit_log(range)
    }

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    ///
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        self
    }
}

impl<Anim: Animation> AnimationMotion for FloModel<Anim> {
    ///
    /// Retrieves the IDs of the motions attached to a particular element
    ///
    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        self.animation.motion().get_motions_for_element(element_id)
    }

    ///
    /// Retrieves the IDs of the elements attached to a particular motion
    ///
    fn get_elements_for_motion(&self, motion_id: ElementId) -> Vec<ElementId> {
        self.animation.motion().get_elements_for_motion(motion_id)
    }

    ///
    /// Retrieves the motion with the specified ID
    ///
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        self.animation.motion().get_motion(motion_id)
    }
}

impl<Anim: 'static+Animation+EditableAnimation> EditableAnimation for FloModel<Anim> {
    fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        Self::process_edits(&edits, &self.size_binding, &self.timeline, &self.frame_edit_counter);
        self.animation.perform_edits(edits);
    }

    ///
    /// Retrieves a sink that can be used to send edits for this animation
    ///
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    ///
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.edit_publisher.sync(|publisher| publisher.republish())
    }

    ///
    /// Assigns a new unique ID for creating a new motion
    ///
    /// This ID will not have been used so far and will not be used again, and can be used as the ID for the MotionElement vector element.
    ///
    fn assign_element_id(&self) -> ElementId {
        self.animation.assign_element_id()
    }

    ///
    /// Flushes any caches this might have (forces reload from data storage)
    ///
    fn flush_caches(&self) {
        self.animation.flush_caches()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use flo_animation::storage::*;
    use futures::executor;

    #[test]
    fn size_command_updates_size_binding() {
        let in_memory_store = InMemoryStorage::new();
        let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let model           = FloModel::new(animation);

        // Initial size is 1980x1080
        assert!(model.size()        == (1920.0, 1080.0));
        assert!(model.size.get()    == (1920.0, 1080.0));

        // Change to 800x600
        executor::block_on(async {
            let mut edit_log = model.edit();
            edit_log.publish(Arc::new(vec![AnimationEdit::SetSize(800.0, 600.0)])).await;
            edit_log.when_empty().await;
            model.when_complete().await;
        });

        // Binding should get changed by this edit
        assert!(model.size()        == (800.0, 600.0));
        assert!(model.size.get()    == (800.0, 600.0));
    }
}
