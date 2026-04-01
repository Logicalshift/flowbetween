use crate::state::*;
use crate::output::*;

use futures::prelude::*;

use flo_stream::*;

///
/// Implementation of the list_files command
///
pub fn list_files<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl 'a+Future<Output=()>+Send {
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
            let full_name = file_manager.display_name_for_path(file.as_path()).unwrap_or("<untitled>".to_string());
            let file_name = format!("#{}#: {}", index, full_name);
            output.publish(Output(file_name)).await;
            output.publish(Output("\n".to_string())).await;

            index += 1;
        }
    }
}
