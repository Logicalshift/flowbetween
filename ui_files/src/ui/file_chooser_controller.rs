use super::file_chooser::*;
use super::file_controller::*;
use super::super::open_file_store::*;

use flo_ui::*;
use flo_binding::*;

use std::sync::*;

///
/// The file chooser controller can be used as a front-end for tablet or web-style applications
/// where there is no file system file chooser.
/// 
pub struct FileChooserController<Chooser: FileChooser> {
    /// None, or the controller for the loaded file
    /// 
    /// (If there's a loaded file, the user is no longer choosing a file and this controller will be hidden)
    loaded_file: Option<Chooser::Controller>,

    /// The file manager used for finding the files to be displayed by this controller
    file_manager: Arc<Chooser::FileManager>,

    /// The cache of open files
    open_file_store: Arc<OpenFileStore<<Chooser::Controller as FileController>::Model>>
}

impl<Chooser: FileChooser> FileChooserController<Chooser> {
    ///
    /// Creates a new file chooser controller
    /// 
    pub fn new(chooser: Chooser) -> FileChooserController<Chooser> {
        // Fetch the file manager and file store from the 
        let file_manager    = chooser.get_file_manager();
        let open_file_store = chooser.get_file_store();

        // Create the chooser controller
        FileChooserController {
            loaded_file:        None,
            file_manager:       file_manager,
            open_file_store:    open_file_store
        }
    }
}

impl<Chooser: FileChooser> Controller for FileChooserController<Chooser> {
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()))
    }

    /// Retrieves the viewmodel for this controller
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> { 
        None 
    }

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> { 
        None
    }

    /// Callback for when a control associated with this controller generates an action
    fn action(&self, _action_id: &str, _action_data: &ActionParameter) { 
    }

    /// Retrieves a resource manager containing the images used in the UI for this controller
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { 
        None
    }

    /// Retrieves a resource manager containing the canvases used in the UI for this controller
    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        None
    }

    /// Called just before an update is processed
    /// 
    /// This is called for every controller every time after processing any actions
    /// that might have occurred.
    fn tick(&self) {

    }
}