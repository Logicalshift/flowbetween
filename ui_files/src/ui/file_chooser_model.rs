use super::file_chooser::*;
use super::file_controller::*;
use super::super::open_file_store::*;

use flo_binding::*;

use std::sync::*;
use std::path::PathBuf;

///
/// Model for the file chooser controller
/// 
pub struct FileChooserModel<Chooser: FileChooser> {
    /// The controller displaying the open file
    pub active_controller: Binding<Option<Arc<Chooser::Controller>>>,

    /// The path of the currently open file
    pub open_file: Binding<Option<PathBuf>>
}

impl<Chooser: FileChooser> FileChooserModel<Chooser> {

    ///
    /// Creates a new file chooser model
    /// 
    pub fn new(chooser: &Chooser) -> FileChooserModel<Chooser> {
        // Initially there is no open file
        let open_file           = bind(None);
        let active_controller   = bind(None);

        FileChooserModel {
            active_controller:  active_controller,
            open_file:          open_file
        }
    }
}
