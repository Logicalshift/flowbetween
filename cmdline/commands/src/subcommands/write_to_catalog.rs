use crate::state::*;
use crate::error::*;
use crate::output::*;
use crate::storage_descriptor::*;

use flo_stream::*;
use flo_animation::storage::*;
use flo_sqlite_storage::*;

use futures::prelude::*;
use std::sync::*;

///
/// Creates a new animation in the catalog with the specified name and writes to it
///
pub fn write_to_catalog<'a>(name: String, output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Don't permit emty names
        let name = if name.len() == 0 { "Auto-generated".to_string() } else { name };

        // Generate a message we'll use when the file opens
        let msg = format!("Created new animation '{}'", name);

        // Create a new animation using the file manager
        let file_manager    = state.file_manager();
        let new_path        = file_manager.create_new_path();
        let storage         = SqliteAnimationStorage::new_with_file(&new_path.clone()).map_err(|_| CommandError::CouldNotCreateAnimation(name.clone()))?;
        let animation       = create_animation_editor(move |commands| storage.get_responses(commands).boxed());
        file_manager.set_display_name_for_path(new_path.as_path(), name.clone());

        // Update the state to point at it
        *state = state.set_output_animation(StorageDescriptor::CatalogName(name), Arc::new(animation));

        // Display the success message when the file is opened
        output.publish(FloCommandOutput::Message(msg)).await;

        Ok(())
    }
}