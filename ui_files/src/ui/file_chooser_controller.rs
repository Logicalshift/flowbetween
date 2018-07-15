use super::file_chooser::*;
use super::file_controller::*;
use super::super::file_manager::*;
use super::super::open_file_store::*;

use flo_ui::*;

use std::sync::*;

///
/// The file chooser controller can be used as a front-end for tablet or web-style applications
/// where there is no file system file chooser.
/// 
pub struct FileChooserController<Chooser: FileChooser> {
    /// None, or the controller for the loaded file
    /// 
    /// (If there's a loaded file, the user is no longer choosinga file and this controller will be hidden)
    loaded_file: Option<Chooser::Controller>,

    /// The file manager used for finding the files to be displayed by this controller
    manager: Arc<Chooser::FileManager>,

    /// The cache of open files
    open_file_store: Arc<OpenFileStore<<Chooser::Controller as FileController>::Model>>
}

impl<Chooser: FileChooser> FileChooserController<Chooser> {

}
