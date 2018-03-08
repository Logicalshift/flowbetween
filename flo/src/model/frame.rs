use binding::*;
use animation::*;

use std::sync::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Duration;

///
/// Provides the model for a layer in the current frame
/// 
#[derive(Clone)]
pub struct FrameLayerModel {
    /// The ID of this layer
    pub layer_id: u64,

    /// The current frmae for this layer
    pub frame: BindRef<Arc<Frame>>,

    /// The underlying binding
    frame_binding: Binding<BoundFrame>
}

/// BoundFrame provides a PartialEq implementation so we can put an Arc<Frame> in a binding
#[derive(Clone)]
struct BoundFrame(Arc<Frame>);

impl PartialEq for BoundFrame {
    fn eq(&self, _other: &BoundFrame) -> bool { false }
}

///
/// The frame model provides bindings for the content of the current frame
/// 
pub struct FrameModel {
    /// The layers in the current frame
    pub layers: BindRef<Vec<FrameLayerModel>>
}

impl FrameModel{
    ///
    /// Creates a new frame model that tracks the specified animation
    /// 
    /// The animation update binding can be updated whenever the frames become
    /// invalidated; the value has no meaning, so any value (for example, the
    /// length of the edit log)
    /// 
    pub fn new<Anim: Animation+'static>(animation: Arc<Anim>, when: BindRef<Duration>, animation_update: BindRef<u64>) -> FrameModel {
        // The hashmap allows us to track frame bindings independently from layer bindings
        let frames: Mutex<HashMap<u64, FrameLayerModel>> = Mutex::new(HashMap::new());

        // Create a computed list of layers (because updates are lazy, this will
        // only update when it's actually read)
        let layers = computed(move || {
            // Claim the frames
            let mut frames = frames.lock().unwrap();

            // We bind to the update so this invalidates whenever the update list changes
            animation_update.get();
            
            // Get the current time
            let time = when.get();

            // Refresh the frames from the animation
            let layer_ids = animation.get_layer_ids();
            
            // Remove layers that aren't in use any more
            let deleted_layers: Vec<_> = layer_ids
                .iter()
                .filter(|layer_id| !frames.contains_key(layer_id))
                .map(|layer_id_ref| *layer_id_ref)
                .collect();
            
            deleted_layers.into_iter().for_each(|deleted_layer_id| { frames.remove(&deleted_layer_id); });

            // Update or generate the frame layer model (something bound to a single layer will get updates for that layer)
            for layer_id in layer_ids.iter() {
                // Generate the frame for this layer
                let layer = animation.get_layer_with_id(*layer_id);

                if let Some(layer) = layer {
                    // Generate the frame
                    // TODO: make this computed! (right now the frame won't update if the time changes)
                    // This also removes the need to set the frame in the event the entry is already occupied, as it will regenerate itself automatically
                    let frame = layer.get_frame_at_time(time);   

                    match frames.entry(*layer_id) {
                        Entry::Occupied(mut occupied) => {
                            // Update the layer with the new frame
                            occupied.get_mut().frame_binding.set(BoundFrame(frame));
                        },

                        Entry::Vacant(mut vacant) => {
                            // Generate a new binding
                            let frame_binding   = bind(BoundFrame(frame));
                            let frame_binding2  = frame_binding.clone();
                            let frame           = BindRef::new(&computed(move || frame_binding2.get().0));

                            vacant.insert(FrameLayerModel {
                                layer_id:       *layer_id,
                                frame:          frame,
                                frame_binding:  frame_binding
                            });
                        }
                    }
                }
            }

            // Generate the final result
            layer_ids.into_iter()
                .map(|layer_id| frames.get(&layer_id).unwrap())
                .cloned()
                .collect()
        });

        // Result is a new FrameModel containing these layers
        FrameModel {
            layers: BindRef::new(&layers)
        }
    }
}
