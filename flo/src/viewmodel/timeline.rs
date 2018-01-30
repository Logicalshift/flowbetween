use super::layer::*;
use super::keyframe::*;

use binding::*;
use animation::*;

use std::sync::*;
use std::ops::Range;
use std::collections::*;
use std::time::Duration;

///
/// ViewModel used for the timeline view
/// 
pub struct TimelineViewModel<Anim: Animation> {
    /// The animation that this view model is for
    animation: Arc<Anim>,

    /// The current time
    pub current_time: Binding<Duration>,

    /// The layers in the timeline
    pub layers: Binding<Vec<LayerViewModel>>,

    /// The ID of the layer currently selected for editing
    pub selected_layer: Binding<Option<u64>>,

    /// The keyframes that occur during a certain time period
    keyframes: Arc<Mutex<HashMap<Range<Duration>, Weak<Binding<Vec<KeyFrameViewModel>>>>>>
}

impl<Anim: Animation> Clone for TimelineViewModel<Anim> {
    fn clone(&self) -> TimelineViewModel<Anim> {
        TimelineViewModel {
            animation:      Arc::clone(&self.animation),
            current_time:   Binding::clone(&self.current_time),
            layers:         Binding::clone(&self.layers),
            selected_layer: Binding::clone(&self.selected_layer),
            keyframes:      Arc::clone(&self.keyframes)
        }
    }
}

impl<Anim: Animation>  TimelineViewModel<Anim> {
    ///
    /// Creates a new timeline viewmodel
    /// 
    pub fn new(animation: Arc<Anim>) -> TimelineViewModel<Anim> {
        // Load the layers from the animation
        let layer_ids   = animation.get_layer_ids();
        let mut layers  = vec![];

        for id in layer_ids {
            let layer = animation.get_layer_with_id(id);
            if let Some(layer) = layer {
                layers.push(LayerViewModel::new(&layer));
            }
        }

        // Create the timeline view model
        TimelineViewModel {
            animation:      animation,
            current_time:   bind(Duration::from_millis(0)),
            layers:         bind(layers),
            selected_layer: bind(Some(0)),
            keyframes:      Arc::new(Mutex::new(HashMap::new()))
        }
    }

    ///
    /// Retrieves a binding that tracks the keyframes in a particular time range
    /// 
    pub fn get_keyframe_binding(&self, when: Range<Duration>) -> Arc<Binding<Vec<KeyFrameViewModel>>> {
        self.tidy_keyframes();
        let mut keyframes = self.keyframes.lock().unwrap();

        // Try to get the existing binding if there is one
        let existing_binding = if let Some(weak_binding) = keyframes.get(&when) {
            weak_binding.upgrade()
        } else {
            None
        };

        if let Some(existing_binding) = existing_binding {
            // Use the existing binding
            existing_binding
        } else {
            // Create a new binding
            let also_when           = when.clone();
            let layers              = self.animation.get_layer_ids();
            let keyframe_viewmodel  = layers.into_iter()
                .map(|layer_id| self.animation.get_layer_with_id(layer_id))
                .filter(|reader| reader.is_some())
                .map(|reader| reader.unwrap())
                .map(move |reader| {
                    let keyframes = reader.get_key_frames_during_time(also_when.clone());
                    (reader, keyframes)
                })
                .flat_map(|(reader, keyframes)| {
                    let layer_id = reader.id();
                    keyframes.map(move |keyframe_time| KeyFrameViewModel {
                        when:       keyframe_time,
                        layer_id:   layer_id
                    })
                })
                .collect();
            
            let new_binding = Arc::new(Binding::new(keyframe_viewmodel));
            keyframes.insert(when, Arc::downgrade(&new_binding));

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
