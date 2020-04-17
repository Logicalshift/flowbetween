use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;
use flo_animation::serializer::*;

use futures::prelude::*;

///
/// Serializes the edits to the output
///
pub fn serialize_edits<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Turn the current edit buffer into a serialized version
        let mut result = String::new();
        serialize_animation_as_edits(&mut result, state.edit_buffer(), "FlowBetween Animation");

        // Send to the output
        output.publish(FloCommandOutput::Output(result)).await;

        Ok(())
    }
}
