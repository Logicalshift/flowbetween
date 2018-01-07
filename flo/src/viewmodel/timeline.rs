use binding::*;

use std::time::Duration;

///
/// ViewModel used for the timeline view
/// 
#[derive(Clone)]
pub struct TimelineViewModel {
    /// The current time
    pub current_time: Binding<Duration>,

    /// The ID of the layer currently selected for editing
    pub selected_layer: Binding<Option<u64>>
}

impl TimelineViewModel {
    pub fn new() -> TimelineViewModel {
        TimelineViewModel {
            current_time:   bind(Duration::from_millis(0)),
            selected_layer: bind(Some(0))
        }
    }
}
