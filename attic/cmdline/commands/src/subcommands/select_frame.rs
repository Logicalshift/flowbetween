use crate::state::*;
use crate::output::*;

use flo_stream::*;

use futures::prelude::*;

///
/// Updates the selected frame to the frame found at the specified time in the specified layer of the input animation
///
pub fn select_frame<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState, layer_id: u64, frame_number: usize) -> impl 'a+Future<Output=()>+Send {
    async move {
        use FloCommandOutput::*;

        // Fetch the frame
        let input_animation = state.input_animation();
        let frame_time      = input_animation.frame_length() * (frame_number as u32);
        let frame           = input_animation.get_layer_with_id(layer_id)
            .map(|layer| layer.get_frame_at_time(frame_time));

        // Display a message indicating success or otherwise
        if frame.is_some() {
            let msg = format!("Frame {}:{} (at T+{}ms)", layer_id, frame_number, frame_time.as_millis());
            output.publish(Message(msg)).await;
        } else {
            let msg = format!("Frame {}:{} was not found", layer_id, frame_number);
            output.publish(Error(msg)).await;
        }

        // Update the state
        *state = state.set_selected_frame(frame);
    }
}
