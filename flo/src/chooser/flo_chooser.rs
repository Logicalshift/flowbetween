use super::consts::*;
use super::super::model::*;
use super::super::editor::*;

use flo_animation::*;
use flo_ui_files::*;
use flo_ui_files::ui::*;
use flo_ui_files::sqlite::*;

use std::sync::*;

///
/// The default file chooser for FlowBetween
///
pub struct FloChooser<Loader: 'static+FileAnimation>
where Loader::NewAnimation: 'static+EditableAnimation {
    /// The file manager managed by this chooser
    file_manager: Arc<SqliteFileManager>,

    /// The shared open file store for this animation
    file_store: Arc<OpenFileStore<FloSharedModel<Loader>>>
}

impl<Loader: 'static+FileAnimation> FloChooser<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    ///
    /// Creates a new chooser
    ///
    pub fn new(loader: Arc<Loader>) -> FloChooser<Loader> {
        // Create the file manager (we use a single default user by default)
        let file_manager = Arc::new(SqliteFileManager::new(APP_NAME, DEFAULT_USER_FOLDER));

        // Create the file store
        let file_store = Arc::new(OpenFileStore::new(loader));

        // Put everything together
        FloChooser {
            file_manager:   file_manager,
            file_store:     file_store
        }
    }
}

impl<Loader: 'static+FileAnimation> FileChooser for FloChooser<Loader>
where Loader::NewAnimation: 'static+EditableAnimation  {
    /// The controller that edits/displays open files
    type Controller = EditorController<Loader>;

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
    fn get_file_store(&self) -> Arc<OpenFileStore<FloSharedModel<Loader>>> {
        Arc::clone(&self.file_store)
    }
}
