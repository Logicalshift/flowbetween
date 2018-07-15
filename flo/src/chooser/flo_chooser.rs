use super::consts::*;

use flo_ui_files::*;
use flo_ui_files::sqlite::*;

use std::sync::*;

///
/// The default file chooser for FlowBetween
/// 
pub struct FloChooser {
    /// The file manager managed by this chooser
    file_manager: Arc<SqliteFileManager>,
}

impl FloChooser {
    ///
    /// Creates a new chooser
    /// 
    pub fn new() -> FloChooser {
        // Create the file manager (we use a single default user by default)
        let file_manager = Arc::new(SqliteFileManager::new(APP_NAME, DEFAULT_USER_FOLDER));

        // Put everything together
        FloChooser {
            file_manager: file_manager
        }
    }
}

/*
impl FileChooser for FloChooser {
    /// The controller that edits/displays open files
    type Controller: FileController;

    /// The file manager that finds paths where files can be located
    type FileManager = SqliteFileManager;

    ///
    /// Retrieves the file manager for this file chooser
    /// 
    fn get_file_manager(&self) -> Arc<Self::FileManager> {
        Arc::clone(&self.file_manager)
    }

    ///
    /// Retrieves the shared file store for this chooser
    /// 
    fn get_file_store(&self) -> Arc<OpenFileStore<<Self::Controller as FileController>::Model>> {

    }
}
*/