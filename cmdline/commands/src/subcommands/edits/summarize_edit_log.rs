use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;

use futures::prelude::*;

///
/// Displays a summary of the contents of the edit log buffer
///
pub fn summarize_edit_log<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        let edit_log = state.edit_buffer();

        // Title message
        output.publish(FloCommandOutput::Message("".to_string())).await;
        output.publish(FloCommandOutput::Message("Summary of edits".to_string())).await;
        output.publish(FloCommandOutput::Message("================".to_string())).await;
        output.publish(FloCommandOutput::Message("".to_string())).await;

        // Total number of items
        let total_number_of_items_msg = format!("Total: {} edit operations", edit_log.len());
        output.publish(FloCommandOutput::Message(total_number_of_items_msg)).await;

        // TODO: item breakdown

        Ok(())
    }
}
