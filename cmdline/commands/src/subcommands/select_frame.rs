use super::super::state::*;

use futures::prelude::*;

use std::time::{Duration};

///
/// Updates the selected frame to the frame found at the specified time in the specified layer of the input animation
///
pub fn select_frame<'a>(state: &'a mut CommandState, layer_id: u64, frame_time: Duration) -> impl 'a+Future<Output=()>+Send {
    async move {
        // Fetch the frame
        let input_animation = state.input_animation();
        let frame           = input_animation.get_layer_with_id(layer_id)
            .map(|layer| layer.get_frame_at_time(frame_time));

        // Update the state
        *state = state.set_selected_frame(frame);
    }
}
