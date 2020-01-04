use super::super::output::*;

use futures::prelude::*;

use flo_stream::*;
use flo_ui_files::*;
use flo_ui_files::sqlite::*;

///
/// Implementations of the list_files command
///
pub fn list_files<'a>(output: &'a mut Publisher<FloCommandOutput>, app_name: String, default_user_folder: String) -> impl 'a+Future<Output=()>+Send {
    async move {
        use self::FloCommandOutput::*;

        let file_manager = SqliteFileManager::new(&app_name, &default_user_folder);

        // Get all the files in the current folder
        let all_files   = file_manager.get_all_files();
        let count       = format!("{} files", all_files.len());
        output.publish(Message(count)).await;
        output.publish(Message("".to_string())).await;

        let mut index = 0;
        for file in all_files {
            let full_name = file_manager.display_name_for_path(file.as_path()).unwrap_or("<untitled>".to_string());
            let file_name = format!("#{}#: {}", index, full_name);
            output.publish(Message(file_name)).await;

            index += 1;
        }
    }
}
