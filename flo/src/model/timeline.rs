use super::layer::*;
use super::keyframe::*;

use binding::*;
use binding::Bound;
use animation::*;

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
    pub layers: Binding<Vec<LayerModel>>,

    /// The ID of the layer currently selected for editing
    pub selected_layer: Binding<Option<u64>>,

    /// The keyframes that occur during a certain time period
    keyframes: Arc<Mutex<HashMap<Range<u32>, Weak<Binding<Vec<KeyFrameModel>>>>>>
}

impl<Anim: Animation> Clone for TimelineModel<Anim> {
    fn clone(&self) -> TimelineModel<Anim> {
        TimelineModel {
            animation:      Arc::clone(&self.animation),
            current_time:   Binding::clone(&self.current_time),
            frame_duration: Binding::clone(&self.frame_duration),
            duration:       Binding::clone(&self.duration),
            layers:         Binding::clone(&self.layers),
            selected_layer: Binding::clone(&self.selected_layer),
            keyframes:      Arc::clone(&self.keyframes)
        }
    }
}

impl<Anim: Animation> TimelineModel<Anim> {
    ///
    /// Creates a new timeline viewmodel
    /// 
    pub fn new(animation: Arc<Anim>) -> TimelineModel<Anim> {
        // Load the layers from the animation
        let layer_ids   = animation.get_layer_ids();
        let mut layers  = vec![];

        for id in layer_ids {
            let layer = animation.get_layer_with_id(id);
            if let Some(layer) = layer {
                layers.push(LayerModel::new(&layer));
            }
        }

        // Read the animation properties
        let duration        = animation.duration();
        let frame_duration  = animation.frame_length();

        // Create the timeline view model
        TimelineModel {
            animation:      animation,
            current_time:   bind(Duration::from_millis(0)),
            duration:       bind(duration),
            frame_duration: bind(frame_duration),
            layers:         bind(layers),
            selected_layer: bind(Some(0)),
            keyframes:      Arc::new(Mutex::new(HashMap::new()))
        }
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
            let frame_duration      = self.frame_duration.get();
            let when                = (frame_duration*frames.start)..(frame_duration*frames.end);
            let layers              = self.animation.get_layer_ids();

            let keyframe_viewmodel  = layers.into_iter()
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

                        KeyFrameModel {
                            when:       keyframe_time,
                            frame:      (frame_time_nanos/frame_duration_nanos) as u32,
                            layer_id:   layer_id
                        }
                    })
                })
                .collect();
            
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
