use super::file_chooser::*;
use super::file_controller::*;
use super::super::file_update::*;
use super::super::file_manager::*;

use flo_binding::*;

use std::sync::*;
use std::ops::Range;
use std::path::{Path, PathBuf};

///
/// Model for a file chooser file
/// 
#[derive(Clone)]
pub struct FileUiModel {
    /// The path to this file
    pub path: Binding<PathBuf>,

    /// The name of this file
    pub name: Binding<String>,
}

impl PartialEq for FileUiModel {
    fn eq(&self, other: &FileUiModel) -> bool {
        self.path.get() == other.path.get() && self.name.get() == other.name.get()
    }
}

///
/// Model for the file chooser controller
/// 
pub struct FileChooserModel<Chooser: FileChooser> {
    /// The controller displaying the open file
    pub active_controller: Binding<Option<Arc<Chooser::Controller>>>,

    /// The shared state in use by the active controller (or none if there is no active controller)
    pub shared_state: Mutex<Option<Arc<<Chooser::Controller as FileController>::Model>>>,

    /// The path of the currently open file
    pub open_file: Binding<Option<PathBuf>>,

    /// The list of files to choose from
    pub file_list: BindRef<Vec<FileUiModel>>,

    /// The range of files to display
    pub file_range: Binding<Range<u32>>
}

impl<Chooser: 'static+FileChooser> FileChooserModel<Chooser> {
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
            shared_state:       Mutex::new(None),
            open_file:          open_file,
            file_list:          file_list,
            file_range:         bind(0..0)
        }
    }

    ///
    /// Creates the file model for a particular path
    /// 
    fn model_for_path(file_manager: &Arc<Chooser::FileManager>, path: &Path) -> FileUiModel {
        let name = file_manager.display_name_for_path(path).unwrap_or("Untitled".to_string());

        FileUiModel {
            path: bind(PathBuf::from(path)),
            name: bind(name)
        }
    }

    ///
    /// Creates the file list binding from a file manager
    /// 
    fn file_list(file_manager: Arc<Chooser::FileManager>) -> BindRef<Vec<FileUiModel>> {
        // Get all of the files from the file manager
        let files = file_manager.get_all_files();
        
        // Create the file models from the paths
        let files: Vec<_> = files.into_iter()
            .map(|path| Self::model_for_path(&file_manager, path.as_path()))
            .collect();
        
        // Bind to updates from the file manager
        let updates = file_manager.update_stream();
        let files   = bind_stream(updates, files, move |mut files, update| {
            match update {
                FileUpdate::NewFile(path) => {
                    files.insert(0, Self::model_for_path(&file_manager, path.as_path()))
                },

                FileUpdate::RemovedFile(path) => {
                    files.retain(|model| model.path.get() != path);
                },

                FileUpdate::SetDisplayName(path, new_name) => {
                    files.iter_mut().for_each(|model| {
                        if model.path.get() == path {
                            model.name.set(new_name.clone());
                        }
                    })
                }
            }
            
            files
        });

        // Generate the final binding
        BindRef::from(files)
    }
}
