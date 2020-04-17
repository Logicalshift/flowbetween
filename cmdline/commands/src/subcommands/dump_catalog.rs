use crate::state::*;
use crate::output::*;
use crate::storage_descriptor::*;

use futures::prelude::*;

use flo_stream::*;
use flo_animation::*;

///
/// Implementation of the list_files command
///
pub fn dump_catalog_as_edits<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl 'a+Future<Output=()>+Send {
    async move {
        use self::FloCommandOutput::*;

        let file_manager = state.file_manager();

        // Get all the files in the current folder
        let all_files   = file_manager.get_all_files();
        let count       = format!("{} files", all_files.len());
        output.publish(Message(count)).await;
        output.publish(Message("".to_string())).await;

        let mut index = 0;
        for file in all_files {
            // Display the file we're working on
            let full_name   = file_manager.display_name_for_path(file.as_path()).unwrap_or("<untitled>".to_string());
            let msg         = format!("Writing file #{} ('{}')", index, full_name);
            output.publish(Message(msg)).await;

            // Open it
            let descriptor  = StorageDescriptor::CatalogNumber(index);
            let file        = descriptor.open_animation(&file_manager);

            let file        = if let Some(file) = file {
                file
            } else {
                output.publish(Error("Could not open file".to_string())).await;
                continue;
            };

            // Create an output file
            let full_name   = full_name.chars().map(|c| {
                match c {
                    '/'     => '-',
                    '\\'    => '-',
                    _       => c
                }
            }).collect::<String>();
            let output_file = format!("{} - {}.edits.flo", index, full_name);
            output.publish(BeginOutput(output_file)).await;

            // Fill the output file with the edits from this animation
            let num_edits       = file.get_num_edits();
            let mut edit_stream = file.read_edit_log(0..num_edits);

            output.publish(FloCommandOutput::StartTask("Serialize edit log".to_string())).await;

            // Read the edits as they arrive from the stream
            let mut edit_count  = 0;
            while let Some(edit) = edit_stream.next().await {
                // Serialize this edit
                let mut serialized_edit = String::new();
                edit.serialize(&mut serialized_edit);
                serialized_edit.push('\n');

                // Send to the output
                output.publish(Output(serialized_edit)).await;

                // Update progress
                edit_count += 1;
                output.publish(FloCommandOutput::TaskProgress(edit_count as f64, num_edits as f64)).await;
            }

            output.publish(FloCommandOutput::FinishTask).await;

            index += 1;
        }
    }
}
