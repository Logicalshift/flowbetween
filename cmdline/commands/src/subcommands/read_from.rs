use crate::state::*;
use crate::error::*;
use crate::output::*;
use crate::storage_descriptor::*;

use flo_stream::*;

use futures::prelude::*;

///
/// The read_from command: changes the input to the location specified by the storage descriptor
///
pub fn read_from<'a>(location: StorageDescriptor, output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Generate a message we'll use when the file opens
        let msg = format!("Opening '{}' for input", location);

        // Load the file using the current state (and update to a new state)
        *state = state.load_input_file(location.clone())
            .ok_or_else(move || {
                // Result is an error if the file cannot be opened
                CommandError::CouldNotOpenAnimation(format!("{}", location))
            })?;

        // Display the success message when the file is opened
        output.publish(FloCommandOutput::Message(msg)).await;

        Ok(())
    }
}