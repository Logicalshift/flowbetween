use super::layer::*;

use binding::*;
use animation::*;

use std::time::Duration;

///
/// ViewModel used for the timeline view
/// 
#[derive(Clone)]
pub struct TimelineViewModel {
    /// The current time
    pub current_time: Binding<Duration>,

    /// The layers in the timeline
    pub layers: Binding<Vec<LayerViewModel>>,

    /// The ID of the layer currently selected for editing
    pub selected_layer: Binding<Option<u64>>
}

impl TimelineViewModel {
    ///
    /// Creates a new timeline viewmodel
    /// 
    pub fn new<Anim: Animation+'static>(animation: &Anim) -> TimelineViewModel {
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
            current_time:   bind(Duration::from_millis(0)),
            layers:         bind(layers),
            selected_layer: bind(Some(0))
        }
    }
}
