use super::layer::*;
use super::keyframe::*;
use super::timeline_updates::*;

use flo_binding::*;
use flo_binding::Bound;
use flo_animation::*;

use futures::*;

use std::sync::*;
use std::ops::Range;
use std::collections::*;
use std::time::Duration;

///
/// ViewModel used for the timeline view
///
pub struct TimelineModel<Anim: Animation> {
    /// The animation that this view model is for
    animation: Arc<Anim>,

    /// The current time
    pub current_time: Binding<Duration>,

    /// The length of a frame in the animation
    pub frame_duration: Binding<Duration>,

    /// The length of the timeline
    pub duration: Binding<Duration>,

    /// The layers in the timeline
    pub layers: BindRef<Vec<LayerModel>>,

    /// The ID of the layer currently selected for editing
    pub selected_layer: Binding<Option<u64>>,

    /// The number of times the canvas has been invalidated
    pub canvas_invalidation_count: Binding<u64>,

    /// The keyframes that occur during a certain time period
    keyframes: Arc<Mutex<HashMap<Range<u32>, Weak<Binding<Vec<KeyFrameModel>>>>>>
}

impl<Anim: Animation> Clone for TimelineModel<Anim> {
    fn clone(&self) -> TimelineModel<Anim> {
        TimelineModel {
            animation:                  Arc::clone(&self.animation),
            current_time:               Binding::clone(&self.current_time),
            frame_duration:             Binding::clone(&self.frame_duration),
            duration:                   Binding::clone(&self.duration),
            layers:                     BindRef::clone(&self.layers),
            selected_layer:             Binding::clone(&self.selected_layer),
            canvas_invalidation_count:  Binding::clone(&self.canvas_invalidation_count),
            keyframes:                  Arc::clone(&self.keyframes)
        }
    }
}

impl<Anim: Animation+'static> TimelineModel<Anim> {
    ///
    /// Creates a new timeline viewmodel
    ///
    pub fn new<EditStream>(animation: Arc<Anim>, edits: EditStream) -> TimelineModel<Anim>
    where EditStream: 'static+Send+Unpin+Stream<Item=Arc<Vec<AnimationEdit>>> {
        let edits = get_timeline_updates(edits);

        // Create the layers binding
        let layers = Self::layers_binding(&animation, edits);

        // Initial selected layer is the first in the list
        let selected_layer = animation.get_layer_ids().into_iter().nth(0);

        // Read the animation properties
        let duration        = animation.duration();
        let frame_duration  = animation.frame_length();

        // Create the timeline view model
        TimelineModel {
            animation:                  animation,
            current_time:               bind(Duration::from_millis(0)),
            duration:                   bind(duration),
            frame_duration:             bind(frame_duration),
            layers:                     layers,
            selected_layer:             bind(selected_layer),
            canvas_invalidation_count:  bind(0),
            keyframes:                  Arc::new(Mutex::new(HashMap::new()))
        }
    }

    ///
    /// Retrieves the layers for an animation
    ///
    fn get_layers(animation: &Arc<Anim>) -> Vec<LayerModel> {
        // Load the layers from the animation
        let layer_ids   = animation.get_layer_ids();
        let mut layers  = vec![];

        for id in layer_ids {
            let layer = animation.get_layer_with_id(id);
            if let Some(layer) = layer {
                layers.push(LayerModel::new(&*layer));
            }
        }

        layers
    }

    ///
    /// Returns a binding for the layers in an animation
    ///
    fn layers_binding<EditStream>(animation: &Arc<Anim>, edits: EditStream) -> BindRef<Vec<LayerModel>>
    where EditStream: 'static+Send+Unpin+Stream<Item=TimelineModelUpdate> {
        // The animation is used to create the initial layer models in the binding
        let animation = Arc::clone(animation);

        // Create a stream filtered to only layer edits
        let layer_edits = edits.filter(|edit| future::ready(edit.is_layer_operation()));

        // Get the initial set of layers
        let layers = Self::get_layers(&animation);

        // Create a stream binding to update them
        let layers = bind_stream(layer_edits, layers, move |layers, edit| {
            use self::TimelineModelUpdate::*;

            // We'll edit the layers in place
            let mut layers = layers;

            match edit {
                TimelineModelUpdate::AddNewLayer(layer_id) => {
                    // Create a new layer model
                    let layer = animation.get_layer_with_id(layer_id);

                    if let Some(layer) = layer {
                        let model = LayerModel::new(&*layer);
                        layers.push(model);
                    }
                },

                RemoveLayer(layer_id) => {
                    // Remove the layer(s?) with the old ID
                    layers.retain(|model| model.id != layer_id)
                },

                _ => { }
            }

            // These are the new layers
            layers
        });

        // Convert to a bindref
        BindRef::from(layers)
    }

    ///
    /// Updates all of the existing keyframe bindings
    ///
    pub fn update_keyframe_bindings(&self) {
        // Iterate through the keyframes
        for (frames, model) in self.keyframes.lock().unwrap().iter_mut() {
            // Only update the model items that are still in use
            if let Some(model) = model.upgrade() {
                // Recreate the keyframes in this range
                let keyframes   = self.get_keyframe_model(frames);

                // Update the model with the new keyframes
                let model       = Binding::clone(&*model);
                model.set(keyframes);
            }
        }
    }

    ///
    /// Causes the canvas to be invalidated
    ///
    pub fn invalidate_canvas(&self) {
        let old_count = self.canvas_invalidation_count.get();
        self.canvas_invalidation_count.set(old_count+1);
    }

    ///
    /// Creates a keyframe model from a frame range
    ///
    fn get_keyframe_model(&self, frames: &Range<u32>) -> Vec<KeyFrameModel> {
        let frame_duration      = self.frame_duration.get();
        let when                = (frame_duration*frames.start)..(frame_duration*frames.end);
        let layers              = self.animation.get_layer_ids();

        let keyframe_model  = layers.into_iter()
            .map(|layer_id|     self.animation.get_layer_with_id(layer_id))
            .filter(|reader|    reader.is_some())
            .map(|reader|       reader.unwrap())
            .map(move |reader| {
                let keyframes = reader.get_key_frames_during_time(when.clone());
                (reader, keyframes)
            })
            .flat_map(|(reader, keyframes)| {
                let layer_id = reader.id();
                keyframes.map(move |keyframe_time| {
                    let frame_duration_nanos: u64   = frame_duration.as_secs() * 1_000_000_000 + (frame_duration.subsec_nanos() as u64);
                    let frame_time_nanos: u64       = keyframe_time.as_secs() * 1_000_000_000 + (keyframe_time.subsec_nanos() as u64);

                    let frame_num                   = (frame_time_nanos+(frame_duration_nanos/2))/frame_duration_nanos;

                    KeyFrameModel {
                        when:       keyframe_time,
                        frame:      frame_num as u32,
                        layer_id:   layer_id
                    }
                })
            })
            .collect();

        keyframe_model
    }

    ///
    /// Retrieves a binding that tracks the keyframes in a particular range of frames
    ///
    pub fn get_keyframe_binding(&self, frames: Range<u32>) -> Arc<Binding<Vec<KeyFrameModel>>> {
        self.tidy_keyframes();
        let mut keyframes = self.keyframes.lock().unwrap();

        // Try to get the existing binding if there is one
        let existing_binding = if let Some(weak_binding) = keyframes.get(&frames) {
            weak_binding.upgrade()
        } else {
            None
        };

        if let Some(existing_binding) = existing_binding {
            // Use the existing binding
            existing_binding
        } else {
            // Create a new binding
            let keyframe_viewmodel = self.get_keyframe_model(&frames);

            let new_binding = Arc::new(Binding::new(keyframe_viewmodel));
            keyframes.insert(frames, Arc::downgrade(&new_binding));

            new_binding
        }
    }

    ///
    /// Removes any defunct keyframe bindings from the keyframes list
    ///
    fn tidy_keyframes(&self) {
        let mut keyframes   = self.keyframes.lock().unwrap();
        let mut dead_times  = vec![];

        // Find the times that are no longer in use
        for (time, ref binding) in keyframes.iter() {
            if binding.upgrade().is_none() {
                dead_times.push(time.clone());
            }
        }

        // Remove them from the hashmap
        dead_times.into_iter().for_each(|dead_time| { keyframes.remove(&dead_time); });
    }
}
