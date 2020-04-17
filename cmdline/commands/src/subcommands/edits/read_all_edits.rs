use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;
use ::desync::*;

use futures::prelude::*;

///
/// The read_all_edits command loads all of the edits from the input animation and stores them in the state buffer
///
pub fn read_all_edits<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Load the input animation
        let input           = Desync::new(state.input_animation());

        // Read the edits from it
        let mut edit_output = output.republish();
        let mut edits       = state.edit_buffer().clone();
        let edits           = input.future(move |input| {
            // Read all of the edits from the input stream
            let num_edits       = input.get_num_edits();
            let mut edit_stream = input.read_edit_log(0..num_edits);

            async move {
                edit_output.publish(FloCommandOutput::StartTask("Read edit log".to_string())).await;

                // Read the edits as they arrive from the stream
                while let Some(edit) = edit_stream.next().await {
                    edits.push(edit);

                    edit_output.publish(FloCommandOutput::TaskProgress(edits.len() as f64, num_edits as f64)).await;
                }

                edit_output.publish(FloCommandOutput::FinishTask).await;

                edits
            }.boxed()
        }).await;

        // Update the edit buffer
        *state = state.set_edit_buffer(edits.unwrap());

        Ok(())
    }
}
