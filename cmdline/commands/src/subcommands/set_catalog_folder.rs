use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;
use flo_ui_files::sqlite::*;

use futures::prelude::*;
use std::sync::*;

///
/// The set_catalog_folder command sets where the animation list will be read from
///
pub fn set_catalog_folder<'a>(new_folder: &'a str, output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Assume we're using the default application path for now
        let new_file_manager = SqliteFileManager::new(new_folder, "default");

        // Update the state
        *state = state.set_file_manager(Arc::new(new_file_manager));

        // Notify the user
        let msg = format!("Set catalog folder to '{}'", new_folder);
        output.publish(FloCommandOutput::Message(msg)).await;

        Ok(())
    }
}

