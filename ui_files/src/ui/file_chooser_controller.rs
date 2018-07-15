use super::file_chooser::*;
use super::file_chooser_model::*;
use super::file_controller::*;
use super::super::open_file_store::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;

use std::sync::*;

const LOGO_HEIGHT: f32  = 256.0;
const NUM_COLUMNS: u32  = 3;
const FILE_WIDTH: f32   = 128.0;
const FILE_HEIGHT: f32  = 80.0;

///
/// The file chooser controller can be used as a front-end for tablet or web-style applications
/// where there is no file system file chooser.
/// 
pub struct FileChooserController<Chooser: FileChooser> {
    /// The model for the controller
    model: FileChooserModel<Chooser>,

    /// The controller that displays the logo UI
    logo_controller: Arc<dyn Controller>,

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
    pub fn new<LogoController: Controller+'static>(chooser: Chooser, logo_controller: LogoController) -> FileChooserController<Chooser> {
        let logo_controller = Arc::new(logo_controller);

        // Fetch the file manager and file store from the chooser
        let file_manager    = chooser.get_file_manager();
        let open_file_store = chooser.get_file_store();

        // Create the model
        let model           = FileChooserModel::new(&chooser);

        // Create the UI
        let ui              = Self::ui(&model);

        // Create the chooser controller
        FileChooserController {
            model:              model,
            logo_controller:    logo_controller,
            ui:                 ui,
            file_manager:       file_manager,
            open_file_store:    open_file_store
        }
    }

    ///
    /// Creates a control representing a file
    /// 
    fn file_ui(file: FileModel) -> Control {
        Control::container()
            .with(Bounds { x1: Position::Start, y1: Position::Start, x2: Position::Offset(FILE_WIDTH), y2: Position::Offset(FILE_HEIGHT) })
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
                Control::scrolling_container()
                    .with(Bounds::fill_all())
                    .with((ActionTrigger::VirtualScroll(8192.0, 512.0), "ScrollFiles"))
                    .with(Scroll::MinimumContentSize(1024.0, 8192.0))
                    .with(vec![
                        // Logo
                        Control::container()
                            .with(Bounds::next_vert(LOGO_HEIGHT))
                            .with(ControlAttribute::Controller("Logo".to_string())),

                        // Divider
                        Control::empty()
                            .with(Bounds::next_vert(8.0)),
                        Control::empty()
                            .with(Bounds::next_vert(1.0))
                            .with(ControlAttribute::Padding((64, 0), (64, 0)))
                            .with(Appearance::Background(Color::Rgba(0.2, 0.2, 0.2, 1.0))),
                        Control::empty()
                            .with(Bounds::next_vert(8.0)),

                        // Buttons
                        Control::container()
                            .with(vec![
                                Control::empty()
                                    .with(Bounds::stretch_horiz(1.0)),
                                Control::button()
                                    .with(vec![Control::label()
                                        .with("+ New file")]),
                                Control::empty()
                                    .with(Bounds::stretch_horiz(1.0))
                            ])
                            .with(Bounds::next_vert(24.0)),

                        // Actual files
                        Control::container()
                            .with(Bounds::fill_vert())
                    ])

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

            // The logo controller is used to display a logo above the list of files
            "Logo" => Some(Arc::clone(&self.logo_controller)),

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