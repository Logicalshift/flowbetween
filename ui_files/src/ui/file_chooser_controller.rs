use super::file_chooser::*;
use super::file_chooser_model::*;
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
    /// The model for the controller
    model: FileChooserModel<Chooser>,

    /// The user interface binding
    ui: BindRef<Control>,

    /// The file manager used for finding the files to be displayed by this controller
    file_manager: Arc<Chooser::FileManager>,

    /// The cache of open files
    open_file_store: Arc<OpenFileStore<<Chooser::Controller as FileController>::Model>>
}

impl<Chooser: FileChooser+'static> FileChooserController<Chooser> {
    ///
    /// Creates a new file chooser controller
    /// 
    pub fn new(chooser: Chooser) -> FileChooserController<Chooser> {
        // Fetch the file manager and file store from the 
        let file_manager    = chooser.get_file_manager();
        let open_file_store = chooser.get_file_store();

        // Create the model
        let model           = FileChooserModel::new(&chooser);

        // Create the UI
        let ui              = Self::ui(&model);

        // Create the chooser controller
        FileChooserController {
            model:              model,
            ui:                 ui,
            file_manager:       file_manager,
            open_file_store:    open_file_store
        }
    }

    ///
    /// Creates the UI binding from the model
    /// 
    fn ui(model: &FileChooserModel<Chooser>) -> BindRef<Control> {
        // Create references to the parts of the model we need
        let controller = model.active_controller.clone();

        // Generate the UI
        let ui = computed(move || {
            let controller = controller.get();

            if controller.is_some() {

                // The UI is just the UI of the main controller
                Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Controller("OpenFile".to_string()))

            } else {
                
                // The UI allows the user to pick a file
                Control::empty()

            }
        });

        // Create a binding from it
        BindRef::from(ui)
    }
}

impl<Chooser: FileChooser+'static> Controller for FileChooserController<Chooser> {
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    /// Retrieves the viewmodel for this controller
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> { 
        None 
    }

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> { 
        match id {
            // The open file controller is whatever the active model points to
            "OpenFile" => {
                self.model.active_controller.get()
                    .map(|controller| {
                        let controller: Arc<dyn Controller+'static> = controller;
                        controller
                    })
            }

            // Default is 'no controller'
            _ => None
        }
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