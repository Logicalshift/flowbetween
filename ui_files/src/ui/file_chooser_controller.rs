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
use std::path::Path;
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

    /// The viewmodel for the controller
    viewmodel: Arc<DynamicViewModel>,

    /// The controller that displays the logo UI
    logo_controller: Arc<dyn Controller>,

    /// The user interface binding
    ui: BindRef<Control>,

    /// The background colour for the controller
    background_color: Binding<Color>,

    /// The file manager used for finding the files to be displayed by this controller
    file_manager: Arc<Chooser::FileManager>,

    /// The cache of open files
    open_file_store: Arc<OpenFileStore<<Chooser::Controller as FileController>::Model>>,
}

impl<Chooser: FileChooser+'static> FileChooserController<Chooser> {
    ///
    /// Creates a new file chooser controller
    ///
    pub fn new<LogoController: Controller+'static>(chooser: Chooser, logo_controller: LogoController) -> FileChooserController<Chooser> {
        let logo_controller     = Arc::new(logo_controller);

        // Fetch the file manager and file store from the chooser
        let file_manager        = chooser.get_file_manager();
        let open_file_store     = chooser.get_file_store();

        // Create the model
        let model               = FileChooserModel::new(&chooser);

        // Set up the viewmodel
        let viewmodel           = DynamicViewModel::new();

        let drag_offset         = model.dragging_offset.clone();
        viewmodel.set_computed("DragX", move || PropertyValue::Float(drag_offset.get().0));
        let drag_offset         = model.dragging_offset.clone();
        viewmodel.set_computed("DragY", move || PropertyValue::Float(drag_offset.get().1));

        let viewmodel           = Arc::new(viewmodel);

        // Create the UI
        let background_color    = bind(Color::Rgba(0.1, 0.1, 0.1, 1.0));
        let ui                  = Self::ui(&model, BindRef::from(background_color.clone()), Arc::clone(&viewmodel));

        // Create the chooser controller
        FileChooserController {
            model:              model,
            viewmodel:          viewmodel,
            logo_controller:    logo_controller,
            ui:                 ui,
            file_manager:       file_manager,
            background_color:   background_color,
            open_file_store:    open_file_store
        }
    }

    ///
    /// Converts a path to a string
    ///
    fn string_for_path(path: &Path) -> String {
        // As a safety measure, we don't allow any directories so only the last path component is used
        let final_component = path.components()
            .last()
            .map(|component| component.as_os_str().to_string_lossy())
            .map(|component| String::from(component));

        final_component.unwrap_or_else(|| String::from(""))
    }

    ///
    /// Binds the UI model for a file to the dynamic view model
    ///
    fn create_viewmodel_for_file(viewmodel: Arc<DynamicViewModel>, file: &FileUiModel) {
        // Files are keyed on their path
        let path            = file.path.get();
        let path_string     = Self::string_for_path(path.as_path());
        let property_name   = format!("Selected-{}", path_string);
        let selected        = file.selected.clone();

        if !viewmodel.has_binding(&property_name) {
            viewmodel.set_computed(&property_name, move || PropertyValue::Bool(selected.get()));
        }
    }

    ///
    /// Changes the background of the file chooser
    ///
    pub fn set_background(&self, new_background: Color) {
        self.background_color.set(new_background)
    }

    ///
    /// Creates a control representing a file
    ///
    fn file_ui(file: &FileUiModel, index: u32, editing_filename_index: BindRef<Option<usize>>, viewmodel: Arc<DynamicViewModel>) -> Control {
        // If the user is editing the filename, then use a textbox instead of the label
        let label = if editing_filename_index.get() == Some(index as usize) {
            Control::container()
                .with(Bounds::next_vert(22.0))
                .with(vec![
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                    Control::text_box()
                        .with(TextAlign::Center)
                        .with(FontWeight::Normal)
                        .with(State::FocusPriority(Property::from(128.0)))
                        .with(Bounds::stretch_horiz(1.0))
                        .with((ActionTrigger::Click, "DoNotClickThrough"))
                        .with((ActionTrigger::EditValue, "SetEditedFilename"))
                        .with((ActionTrigger::CancelEdit, "CancelEditingFilename"))
                        .with((ActionTrigger::Dismiss, "StopEditingFilename"))
                        .with((ActionTrigger::SetValue, "StopEditingFilename"))
                        .with(file.name.get()),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                ])
        } else {
            Control::label()
                .with(TextAlign::Center)
                .with(FontWeight::Normal)
                .with(Bounds::next_vert(22.0))
                .with(file.name.get())
                .with((ActionTrigger::Click, format!("EditName-{}", index)))
                .with((ActionTrigger::Drag, format!("Drag-{}", index)))
        };

        let path        = file.path.get();
        let path_string = Self::string_for_path(path.as_path());

        // Make sure the file is created in the viewmodel
        Self::create_viewmodel_for_file(viewmodel, file);

        // Control consists of a panel showing a preview of the file and a label showing the 'filename'
        Control::container()
            .with(vec![
                Control::empty()
                    .with(Bounds::stretch_vert(1.0))
                    .with(Appearance::Background(Color::Rgba(0.0, 0.6, 0.9, 1.0)))
                    .with((ActionTrigger::Click, format!("Open-{}", index)))
                    .with((ActionTrigger::Drag, format!("Drag-{}", index))),
                Control::empty()
                    .with(Bounds::next_vert(2.0)),
                label,
                Control::check_box()
                    .with(ControlAttribute::ZIndex(1))
                    .with(State::Value(Property::Bind(format!("Selected-{}", path_string))))
                    .with((ActionTrigger::SetValue, format!("SetSelect-{}", index)))
                    .with(Bounds { x1: Position::At(2.0), y1: Position::At(2.0), x2: Position::At(22.0), y2: Position::At(22.0) })
            ])
            .with(ControlAttribute::Padding((2, 2), (2, 2)))
    }

    ///
    /// Creates the UI binding from the model
    ///
    fn ui(model: &FileChooserModel<Chooser>, background: BindRef<Color>, viewmodel: Arc<DynamicViewModel>) -> BindRef<Control> {
        // Create references to the parts of the model we need
        let controller              = model.active_controller.clone();
        let file_list               = model.file_list.clone();
        let file_range              = model.file_range.clone();
        let dragging_file           = model.dragging_file.clone();
        let drag_after_index        = model.drag_after_index.clone();
        let editing_filename_index  = model.editing_filename_index.clone();
        let selected_file_count     = model.selected_file_count.clone();
        let confirming_deletion     = model.confirming_deletion.clone();

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
                let file_range          = file_range.get();
                let file_list           = file_list.get();
                let dragging_file       = dragging_file.get();
                let drag_after_index    = drag_after_index.get();

                let mut files           = file_range.into_iter()
                    .filter_map(|file_index| file_list.get(file_index as usize).map(|file| (file_index, file)))
                    .flat_map(|(file_index, file_model)| {
                        let editing_filename_index = BindRef::from(editing_filename_index.clone());

                        let row     = file_index / NUM_COLUMNS;
                        let column  = file_index % NUM_COLUMNS;
                        let x       = (column as f32) * FILE_WIDTH;
                        let y       = (row as f32) * FILE_HEIGHT;

                        if dragging_file == Some(file_index as usize) {
                            // File that is being dragged has a high z-index and moves with the drag position
                            vec![
                                Control::empty()
                                    .with(Bounds {
                                        x1: Position::At(x+4.0),
                                        y1: Position::At(y+2.0),
                                        x2: Position::At(x+FILE_WIDTH-4.0),
                                        y2: Position::At(y+FILE_HEIGHT-26.0)
                                    })
                                    .with(Appearance::Background(Color::Rgba(0.0, 0.0, 0.4, 0.1))),

                                Self::file_ui(file_model, file_index, editing_filename_index, Arc::clone(&viewmodel))
                                    .with(Bounds {
                                        x1: Position::Floating(Property::bound("DragX"), x),
                                        y1: Position::Floating(Property::bound("DragY"), y),
                                        x2: Position::Floating(Property::bound("DragX"), x+FILE_WIDTH),
                                        y2: Position::Floating(Property::bound("DragY"), y+FILE_HEIGHT)
                                    })
                                    .with(ControlAttribute::ZIndex(10))
                                    .with(Appearance::Background(Color::Rgba(0.0, 0.0, 0.0, 0.15)))
                            ]
                        } else if drag_after_index == Some((file_index as i64)-1) {
                            // Going to insert before this file
                            vec![
                                Self::file_ui(file_model, file_index, editing_filename_index, Arc::clone(&viewmodel))
                                    .with(Bounds {
                                        x1: Position::At(x),
                                        y1: Position::At(y),
                                        x2: Position::At(x+FILE_WIDTH),
                                        y2: Position::At(y+FILE_HEIGHT) })
                                    .with(ControlAttribute::ZIndex(0)),

                                Control::empty()
                                    .with(Bounds {
                                        x1: Position::At(x-1.0),
                                        y1: Position::At(y),
                                        x2: Position::At(x+1.0),
                                        y2: Position::At(y+FILE_HEIGHT)
                                    })
                                    .with(Appearance::Background(Color::Rgba(0.0, 0.6, 0.8, 0.9)))
                                    .with(ControlAttribute::ZIndex(1))
                            ]
                        } else {
                            // File that is static and not being dragged
                            vec![
                                Self::file_ui(file_model, file_index, editing_filename_index, Arc::clone(&viewmodel))
                                    .with(Bounds {
                                        x1: Position::At(x),
                                        y1: Position::At(y),
                                        x2: Position::At(x+FILE_WIDTH),
                                        y2: Position::At(y+FILE_HEIGHT) })
                                    .with(ControlAttribute::ZIndex(0))
                            ]
                        }
                    })
                    .collect::<Vec<_>>();

                // Add an extra indicator if the drag file index is at the end
                if drag_after_index == Some(file_list.len() as i64 - 1) {
                    let file_index  = drag_after_index.unwrap() as u32 + 1;
                    let row         = file_index / NUM_COLUMNS;
                    let column      = file_index % NUM_COLUMNS;
                    let x           = (column as f32) * FILE_WIDTH;
                    let y           = (row as f32) * FILE_HEIGHT;

                    files.push(Control::empty()
                        .with(Bounds {
                            x1: Position::At(x-1.0),
                            y1: Position::At(y),
                            x2: Position::At(x+1.0),
                            y2: Position::At(y+FILE_HEIGHT)
                        })
                        .with(Appearance::Background(Color::Rgba(0.0, 0.6, 0.8, 0.9)))
                        .with(ControlAttribute::ZIndex(1)));
                }

                // If any files are selected, we display a set of selected file controls
                let selected_file_count = selected_file_count.get();
                let confirming_deletion = confirming_deletion.get();
                let selected_file_controls = if selected_file_count > 0 {
                    // Some files are selected: display the controls
                    vec![
                        Control::container()
                            .with(Bounds {
                                x1: Position::At(8.0),
                                x2: Position::At(192.0),
                                y1: Position::At(8.0),
                                y2: Position::At(88.0)
                            })
                            .with(ControlAttribute::Padding((4, 4), (4, 4)))
                            .with(vec![
                                Control::button()
                                    .with(Bounds::next_vert(32.0))
                                    .with(vec![Control::label()
                                        .with(Bounds::fill_all())
                                        .with(TextAlign::Center)
                                        .with("Clear selection")
                                    ])
                                    .with((ActionTrigger::Click, "ClearSelection")),

                                Control::empty()
                                    .with(Bounds::next_vert(8.0)),

                                if confirming_deletion {
                                    Control::button()
                                        .with(Bounds::next_vert(32.0))
                                        .with(vec![Control::label()
                                            .with(Bounds::fill_all())
                                            .with(TextAlign::Center)
                                            .with(&format!("Confirm: delete {} file{}!", selected_file_count, if selected_file_count == 1 { "" } else { "s" }))
                                        ])
                                        .with((ActionTrigger::Click, "DeleteSelectedFiles"))
                                        .with((ActionTrigger::Dismiss, "DismissDeleteSelectedFiles"))
                                        .with(Appearance::Background(Color::Rgba(1.0, 0.0, 0.0, 0.3)))
                                } else {
                                    Control::button()
                                        .with(Bounds::next_vert(32.0))
                                        .with(vec![Control::label()
                                            .with(Bounds::fill_all())
                                            .with(TextAlign::Center)
                                            .with(&format!("Delete {} selected file{}", selected_file_count, if selected_file_count == 1 { "" } else { "s" }))
                                        ])
                                        .with((ActionTrigger::Click, "ConfirmDeleteSelectedFiles"))
                                    }
                            ])
                            .with(Appearance::Background(Color::Rgba(0.0, 0.0, 0.0, 0.3)))
                            .with(Scroll::Fix(FixedAxis::Vertical))
                            .with(ControlAttribute::ZIndex(5))
                    ]
                } else {
                    // The UI has no file selection controls
                    vec![]
                };

                // Work out the height of the container
                let num_rows    = ((file_list.len() as i32)-1) / (NUM_COLUMNS as i32) + 1;
                let height      = LOGO_HEIGHT + 8.0 + 24.0 + FILE_HEIGHT * (num_rows as f32);

                // The UI allows the user to pick a file
                Control::scrolling_container()
                    .with(Appearance::Background(background.get()))
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
                    ]
                    .into_iter()
                    .chain(selected_file_controls)
                    .collect::<Vec<_>>()
                    )

            }
        });

        // Create a binding from it
        BindRef::from(ui)
    }

    ///
    /// Stops the filename editing process and updates the name of the file
    ///
    pub fn stop_editing_filename(&self) {
        if let Some(edited_file_index) = self.model.editing_filename_index.get() {
            // Stop editing the filename
            self.model.editing_filename_index.set(None);

            // Fetch the file that was edited and its new name
            let file_model      = &self.model.file_list.get()[edited_file_index];
            let file_path       = file_model.path.get();
            let new_filename    = self.model.edited_filename.get();

            // Send to the file manager for an update
            self.file_manager.set_display_name_for_path(&file_path.as_path(), new_filename);
        }
    }
}

impl<Chooser: FileChooser+'static> Controller for FileChooserController<Chooser> {
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    /// Retrieves the viewmodel for this controller
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.viewmodel.clone())
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
                self.model.file_range.set(top..bottom);
            },

            ("CreateNewFile", _) => {
                // Stop any editing
                self.stop_editing_filename();

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

                self.file_manager.set_display_name_for_path(new_file.as_path(), new_name.clone());

                // Edit the name
                self.model.edited_filename.set(new_name);
                self.model.editing_filename_index.set(Some(0));
            },

            ("ClearSelection", _) => {
                self.model.confirming_deletion.set(false);
                self.model.file_list.get()
                    .iter()
                    .for_each(|file_model| file_model.selected.set(false));
            },

            ("ConfirmDeleteSelectedFiles", _) => {
                self.model.confirming_deletion.set(true);
            },

            ("DeleteSelectedFiles", _) => {
                if self.model.confirming_deletion.get() == true {
                    // Event arrived when the model was set up as expected
                    self.model.confirming_deletion.set(false);

                    // Send delete messages for all selected files
                    let selected_paths = self.model.file_list.get()
                        .iter()
                        .filter(|file_model| file_model.selected.get())
                        .map(|file_model| file_model.path.get())
                        .collect::<Vec<_>>();

                    selected_paths.into_iter()
                        .for_each(|path_to_delete| {
                            self.file_manager.delete_path(path_to_delete.as_path());
                        });
                }
            },

            ("DismissDeleteSelectedFiles", _) => {
                self.model.confirming_deletion.set(false);
            },

            ("CancelEditingFilename", _) => {
                // Just unset the editing index without storing the edited value
                self.model.editing_filename_index.set(None);
            },

            ("StopEditingFilename", _) => {
                // User dismissed the filename editor
                self.stop_editing_filename();
            },

            ("SetEditedFilename", ActionParameter::Value(PropertyValue::String(new_filename))) => {
                // Set the edited version of the filename in the model
                self.model.edited_filename.set(new_filename.clone());
            },

            (action, action_parameter) => {
                if action.starts_with("Open-") {

                    // Finish up any filename editing that might be occurring
                    self.stop_editing_filename();

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
                    self.model.open_file.set(Some(path));
                    self.model.active_controller.set(Some(new_controller));

                } else if action.starts_with("SetSelect-") {

                    if let ActionParameter::Value(PropertyValue::Bool(is_selected)) = action_parameter {
                        // Get the index of the file being selected
                        let (_, file_index) = action.split_at("SetSelect-".len());
                        let file_index      = usize::from_str_radix(file_index, 10).unwrap();
                        let file_model      = &self.model.file_list.get()[file_index];

                        // Set as the editing file
                        file_model.selected.set(*is_selected);
                    }

                } else if action.starts_with("EditName-") {

                    // Get the index of the file being edited
                    let (_, file_index) = action.split_at("EditName-".len());
                    let file_index      = usize::from_str_radix(file_index, 10).unwrap();
                    let file_model      = &self.model.file_list.get()[file_index];

                    // Set as the editing file
                    self.model.edited_filename.set(file_model.name.get());
                    self.model.editing_filename_index.set(Some(file_index));

                } else if action.starts_with("Drag-") {

                    // Stops file editing
                    self.stop_editing_filename();

                    // Get the index of the file being dragged
                    let (_, file_index) = action.split_at("Drag-".len());
                    let file_index      = usize::from_str_radix(file_index, 10).unwrap();

                    // Action depends on the parameter
                    match action_parameter {
                        ActionParameter::Drag(DragAction::Start, _, _) => {
                            self.model.dragging_offset.set((0.0, 0.0));
                        },

                        ActionParameter::Drag(DragAction::Finish, _, _) => {
                            let drag_after_index = self.model.drag_after_index.get();

                            // Move the file if there's a drag index
                            if let Some(drag_after_index) = drag_after_index {
                                let files       = self.model.file_list.get();
                                let src_path    = files[file_index].path.get();

                                if drag_after_index < 0 {
                                    // Drag to beginning
                                    self.file_manager.order_path_after(src_path.as_path(), None);
                                } else if drag_after_index < files.len() as i64 {
                                    // Drag after specific file
                                    let tgt_path = files[drag_after_index as usize].path.get();
                                    self.file_manager.order_path_after(src_path.as_path(), Some(tgt_path.as_path()));
                                }
                            }

                            // Clear the drag operation
                            self.model.dragging_file.set(None);
                            self.model.drag_after_index.set(None);
                        },

                        ActionParameter::Drag(DragAction::Cancel, _, _) => {
                            self.model.dragging_file.set(None);
                            self.model.drag_after_index.set(None);
                        },

                        ActionParameter::Drag(DragAction::Drag, (from_x, from_y), (to_x, to_y)) => {
                            if (from_x-to_x).abs() > 6.0 || (from_y-to_y).abs() > 6.0 {
                                self.model.dragging_file.set(Some(file_index as usize));
                            }
                            self.model.dragging_offset.set(((to_x-from_x) as f64, (to_y-from_y) as f64));

                            // Work out the distance the file has been dragged in rows and columns
                            let origin_col  = (file_index % (NUM_COLUMNS as usize)) as i64;
                            let origin_row  = (file_index / (NUM_COLUMNS as usize)) as i64;

                            let offset_x    = to_x-(FILE_WIDTH/2.0);
                            let offset_y    = to_y;

                            let offset_cols = (offset_x / FILE_WIDTH).floor() as i64;
                            let offset_rows = (offset_y / FILE_HEIGHT).floor() as i64;

                            // Work out the target position
                            let target_col  = origin_col + offset_cols;
                            let target_row  = origin_row + offset_rows;

                            let target_idx  = target_col + target_row*(NUM_COLUMNS as i64);
                            let max_idx     = self.model.file_list.get().len();

                            if target_col >= -1 && target_col < (NUM_COLUMNS as i64) && target_idx >= -1 && target_idx < (max_idx as i64) {
                                // Valid target index
                                self.model.drag_after_index.set(Some(target_idx));
                            } else {
                                // No target index
                                self.model.drag_after_index.set(None);
                            }
                        },

                        _ => {}
                    }
                }
            }
        }
    }

    /// Retrieves a resource manager containing the images used in the UI for this controller
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        None
    }
}
