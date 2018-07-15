use super::file_chooser::*;
use super::super::file_manager::*;

use flo_binding::*;

use std::sync::*;
use std::path::PathBuf;

///
/// Model for a file chooser file
/// 
#[derive(Clone)]
pub struct FileModel {
    /// The path to this file
    pub path: BindRef<PathBuf>,

    /// The name of this file
    pub name: BindRef<String>,
}

impl PartialEq for FileModel {
    fn eq(&self, other: &FileModel) -> bool {
        self.path.get() == other.path.get() && self.name.get() == other.name.get()
    }
}

///
/// Model for the file chooser controller
/// 
pub struct FileChooserModel<Chooser: FileChooser> {
    /// The controller displaying the open file
    pub active_controller: Binding<Option<Arc<Chooser::Controller>>>,

    /// The path of the currently open file
    pub open_file: Binding<Option<PathBuf>>,

    /// The list of files to choose from
    pub file_list: BindRef<Vec<FileModel>>
}

impl<Chooser: FileChooser> FileChooserModel<Chooser> {
    ///
    /// Creates a new file chooser model
    /// 
    pub fn new(chooser: &Chooser) -> FileChooserModel<Chooser> {
        // Initially there is no open file
        let open_file           = bind(None);
        let active_controller   = bind(None);

        // Create the actual file list model
        let file_list           = Self::file_list(chooser.get_file_manager());

        // Combine into the final file chooser model
        FileChooserModel {
            active_controller:  active_controller,
            open_file:          open_file,
            file_list:          file_list
        }
    }

    ///
    /// Creates the file list binding from a file manager
    /// 
    fn file_list(file_manager: Arc<Chooser::FileManager>) -> BindRef<Vec<FileModel>> {
        // Get all of the files from the file manager
        let files = file_manager.get_all_files();
        
        // Create the file models from the paths
        let files = files.into_iter()
            .map(|path| {
                let name = file_manager.display_name_for_path(path.as_path()).unwrap_or("Untitled".to_string());

                FileModel {
                    path: BindRef::from(bind(path)),
                    name: BindRef::from(bind(name))
                }
            })
            .collect();

        // Generate the final binding
        BindRef::from(bind(files))
    }
}
