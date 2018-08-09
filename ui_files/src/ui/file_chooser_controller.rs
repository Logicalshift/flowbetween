use super::file_chooser::*;
use super::file_chooser_model::*;
use super::file_controller::*;
use super::super::file_model::*;
use super::super::file_manager::*;
use super::super::open_file_store::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;

use std::sync::*;
use std::collections::HashSet;

const LOGO_HEIGHT: f32      = 256.0;
const NUM_COLUMNS: u32      = 3;
const FILE_WIDTH: f32       = 256.0;
const FILE_HEIGHT: f32      = 180.0;
const VIRTUAL_HEIGHT: f32   = 512.0;

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
    fn file_ui(file: &FileUiModel, index: u32) -> Control {
        Control::container()
            .with(vec![
                Control::empty()
                    .with(Bounds::stretch_vert(1.0))
                    .with(Appearance::Background(Color::Rgba(0.0, 0.6, 0.9, 1.0)))
                    .with(ControlAttribute::Padding((16, 2), (16, 2))),
                Control::label()
                    .with(TextAlign::Center)
                    .with(Bounds::next_vert(24.0))
                    .with(file.name.get())
            ])
            .with(ControlAttribute::Padding((2, 2), (2, 2)))
            .with((ActionTrigger::Click, format!("Open-{}", index)))
    }

    ///
    /// Creates the UI binding from the model
    /// 
    fn ui(model: &FileChooserModel<Chooser>) -> BindRef<Control> {
        // Create references to the parts of the model we need
        let controller  = model.active_controller.clone();
        let file_list   = model.file_list.clone();
        let file_range  = model.file_range.clone();

        // Generate the UI
        let ui = computed(move || {
            let controller = controller.get();

            if controller.is_some() {

                // The UI is just the UI of the main controller
                Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Controller("OpenFile".to_string()))

            } else {
                
                // The file controls that are currently on-screen (virtualised)
                let file_range  = file_range.get();
                let file_list   = file_list.get();

                let files       = file_range.into_iter()
                    .filter_map(|file_index| file_list.get(file_index as usize).map(|file| (file_index, file)))
                    .map(|(file_index, file_model)| {
                        let row     = file_index / NUM_COLUMNS;
                        let column  = file_index % NUM_COLUMNS;
                        let x       = (column as f32) * FILE_WIDTH;
                        let y       = (row as f32) * FILE_HEIGHT;

                        Self::file_ui(file_model, file_index)
                            .with(Bounds { x1: Position::At(x), y1: Position::At(y), x2: Position::At(x+FILE_WIDTH), y2: Position::At(y+FILE_HEIGHT) })
                    })
                    .collect::<Vec<_>>();
                
                // Work out the height of the container
                let num_rows    = ((file_list.len() as i32)-1) / (NUM_COLUMNS as i32) + 1;
                let height      = LOGO_HEIGHT + 8.0 + 24.0 + FILE_HEIGHT * (num_rows as f32);

                // The UI allows the user to pick a file
                Control::scrolling_container()
                    .with(Bounds::fill_all())
                    .with((ActionTrigger::VirtualScroll(8192.0, VIRTUAL_HEIGHT), "ScrollFiles"))
                    .with(Scroll::MinimumContentSize((NUM_COLUMNS as f32)*FILE_WIDTH, height))
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
                                    .with(Bounds::next_horiz(120.0))
                                    .with(vec![Control::label()
                                        .with(Bounds::fill_all())
                                        .with(TextAlign::Center)
                                        .with("+ New file")])
                                        .with((ActionTrigger::Click, "CreateNewFile")),
                                Control::empty()
                                    .with(Bounds::stretch_horiz(1.0))
                            ])
                            .with(Bounds::next_vert(32.0)),
                        
                        Control::empty()
                            .with(Bounds::next_vert(4.0)),

                        // Actual files
                        Control::container()
                            .with(Bounds::fill_vert())
                            .with(vec![
                                Control::empty()
                                    .with(Bounds::stretch_horiz(1.0)),
                                Control::container()
                                    .with(Bounds::next_horiz(FILE_WIDTH * (NUM_COLUMNS as f32)))
                                    .with(files)
                                    .with(Appearance::Foreground(Color::Rgba(1.0, 1.0, 1.0, 1.0)))
                                    .with(Font::Size(11.0))
                                    .with(Font::Weight(FontWeight::Light)),
                                Control::empty()
                                    .with(Bounds::stretch_horiz(1.0)),
                            ])
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
            },

            // The logo controller is used to display a logo above the list of files
            "Logo" => Some(Arc::clone(&self.logo_controller)),

            // Default is 'no controller'
            _ => None
        }
    }

    /// Callback for when a control associated with this controller generates an action
    fn action(&self, action_id: &str, action_data: &ActionParameter) { 
        match (action_id, action_data) {
            ("ScrollFiles", ActionParameter::VirtualScroll((_x, y), (_width, height))) => {
                // Get the position of the files
                let top     = (*y as f32) * VIRTUAL_HEIGHT;
                let bottom  = ((y+height) as f32) * VIRTUAL_HEIGHT;

                // Correct for logo position
                let top     = top - LOGO_HEIGHT;
                let bottom  = bottom - LOGO_HEIGHT;

                // Get the file range
                let top     = (top/FILE_HEIGHT - 1.0).floor().max(0.0);
                let bottom  = (bottom/FILE_HEIGHT + 2.0).ceil().max(0.0);

                // Update the model
                let top     = top as u32;
                let bottom  = bottom as u32;
                let top     = top * NUM_COLUMNS;
                let bottom  = bottom * NUM_COLUMNS;
                self.model.file_range.clone().set(top..bottom);
            },

            ("CreateNewFile", _) => {
                // Create a new file in the file manager
                let new_file = self.file_manager.create_new_path();

                // Give it a unique name
                let all_files       = self.file_manager.get_all_files();
                let used_names      = all_files.into_iter().filter_map(|path| self.file_manager.display_name_for_path(path.as_path())).collect::<HashSet<_>>();

                let mut new_name    = String::from("New file");
                let mut name_index  = 0;

                while used_names.contains(&new_name) {
                    name_index += 1;
                    new_name = format!("New file ({})", name_index);
                }

                self.file_manager.set_display_name_for_path(new_file.as_path(), new_name);
            },

            (action, _) => { 
                if action.starts_with("Open-") {
                    // Get the index of the file being opened
                    let (_, file_index) = action.split_at("Open-".len());
                    let file_index      = usize::from_str_radix(file_index, 10).unwrap();

                    // ... and the file itself
                    let file_model      = &self.model.file_list.get()[file_index];
                    let path            = file_model.path.get();

                    // Create a new controller for the file
                    let shared_state    = self.open_file_store.open_shared(path.as_path());
                    let instance_state  = shared_state.new_instance();
                    let new_controller  = Chooser::Controller::open(instance_state);
                    let new_controller  = Arc::new(new_controller);

                    // Set as the main controller
                    *self.model.shared_state.lock().unwrap() = Some(shared_state);
                    self.model.open_file.clone().set(Some(path));
                    self.model.active_controller.clone().set(Some(new_controller));
                }
            }
        }
    }

    /// Retrieves a resource manager containing the images used in the UI for this controller
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { 
        None
    }
}