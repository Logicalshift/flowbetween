use super::file_controller::*;
use super::super::file_manager::*;
use super::super::open_file_store::*;

use std::sync::*;

///
/// The file chooser trait is implemented by structs that describe a file chooser
///
pub trait FileChooser {
    /// The controller that edits/displays open files
    type Controller: FileController;

    /// The file manager that finds paths where files can be located
    type FileManager: FileManager;

    ///
    /// Retrieves the file manager for this file chooser
    ///
    fn get_file_manager(&self) -> Arc<Self::FileManager>;

    ///
    /// Retrieves the shared file store for this chooser
    ///
    fn get_file_store(&self) -> Arc<OpenFileStore<<Self::Controller as FileController>::Model>>;
}
