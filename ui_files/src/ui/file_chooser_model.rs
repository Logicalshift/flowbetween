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

    /// Whether or not this file is selected
    pub selected: Binding<bool>
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
    pub file_list: BindRef<Arc<Vec<FileUiModel>>>,

    /// The index of the file whose name is being edited
    pub editing_filename_index: Binding<Option<usize>>,

    /// The current name of the file after editing
    pub edited_filename: Binding<String>,

    /// The index of the file that we'll move the current file after if it is dropped here
    pub drag_after_index: Binding<Option<i64>>,

    /// The file in the index that is being dragged (or None if no file is being dragged)
    pub dragging_file: Binding<Option<usize>>,

    /// The X, Y offset for the file that's being dragged
    pub dragging_offset: Binding<(f64, f64)>,

    /// The range of files to display
    pub file_range: Binding<Range<u32>>,

    /// The number of files that have been selected by the user
    pub selected_file_count: BindRef<usize>,

    /// True if we're confirming a deletion request
    pub confirming_deletion: Binding<bool>
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

        // ... and the value indicating if any file is selected
        let selected_file_count = Self::selected_file_count(file_list.clone());

        // Combine into the final file chooser model
        FileChooserModel {
            active_controller:      active_controller,
            shared_state:           Mutex::new(None),
            dragging_file:          bind(None),
            drag_after_index:       bind(None),
            editing_filename_index: bind(None),
            edited_filename:        bind(String::from("")),
            dragging_offset:        bind((0.0, 0.0)),
            open_file:              open_file,
            file_list:              file_list,
            file_range:             bind(0..0),
            selected_file_count:    selected_file_count,
            confirming_deletion:    bind(false)
        }
    }

    ///
    /// Creates the file model for a particular path
    ///
    fn model_for_path(file_manager: &Arc<Chooser::FileManager>, path: &Path) -> FileUiModel {
        let name = file_manager.display_name_for_path(path).unwrap_or("Untitled".to_string());

        FileUiModel {
            path:       bind(PathBuf::from(path)),
            name:       bind(name),
            selected:   bind(false)
        }
    }

    ///
    /// Creates the file list binding from a file manager
    ///
    fn file_list(file_manager: Arc<Chooser::FileManager>) -> BindRef<Arc<Vec<FileUiModel>>> {
        // Get all of the files from the file manager
        let files = file_manager.get_all_files();

        // Create the file models from the paths
        let files: Vec<_> = files.into_iter()
            .map(|path| Self::model_for_path(&file_manager, path.as_path()))
            .collect();
        let files = Arc::new(files);

        // Bind to updates from the file manager
        let updates = file_manager.update_stream();
        let files   = bind_stream(updates, files, move |files, update| {
            let mut files = (*files).clone();

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
                },

                FileUpdate::ChangedOrder(path, after) => {
                    // Find the file being moved
                    let file_idx = files.iter().enumerate()
                        .filter(|(_idx, model)| model.path.get() == path)
                        .map(|(idx, _model)| idx)
                        .nth(0);

                    if let Some(file_idx) = file_idx {
                        // Remove the file from the list
                        let moving_file = files.remove(file_idx);

                        // Find the 'after' index
                        let after_idx = after.and_then(|after| files.iter().enumerate()
                            .filter(|(_idx, model)| model.path.get() == after)
                            .map(|(idx, _model)| idx)
                            .nth(0));

                        if let Some(after_idx) = after_idx {
                            // Move after the 'after' file
                            files.insert(after_idx+1, moving_file);
                        } else {
                            // Move to the beginning
                            files.insert(0, moving_file);
                        }
                    }
                }
            }

            Arc::new(files)
        });

        // Generate the final binding
        BindRef::from(files)
    }

    ///
    /// Returns a binding that sets itself to true if any file is selected in the file list
    ///
    fn selected_file_count(file_list: BindRef<Arc<Vec<FileUiModel>>>) -> BindRef<usize> {
        let any_file_selected = computed(move || {
            let file_list = file_list.get();

            file_list.iter().filter(|file| file.selected.get()).count()
        });

        BindRef::from(any_file_selected)
    }
}
