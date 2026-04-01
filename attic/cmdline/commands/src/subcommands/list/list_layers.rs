use crate::state::*;
use crate::output::*;

use futures::prelude::*;

use flo_stream::*;

///
/// Writes out a list of layers to the output
///
pub fn list_layers<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl 'a+Future<Output=()>+Send {
    async move {
        use self::FloCommandOutput::*;

        let input_animation = state.input_animation();

        // Retrieve all the layers for the animation
        let layer_ids       = input_animation.get_layer_ids();

        output.publish(Message("Animation layers:".to_string())).await;

        for layer_id in layer_ids {
            if let Some(layer) = input_animation.get_layer_with_id(layer_id) {
                // Display information on this layer
                let layer_info = format!("  Layer ({:02}): {}", layer_id, layer.name().unwrap_or("No name".to_string()));
                output.publish(Message(layer_info)).await;
            } else {
                // Layer is in the list but not actually present in the animation
                let layer_info = format!("  Layer ({:02}) - missing", layer_id);
                output.publish(Message(layer_info)).await;
            }
        }
    }
}
