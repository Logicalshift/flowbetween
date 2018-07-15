use super::menu_controller::*;
use super::canvas_controller::*;
use super::toolbox_controller::*;
use super::timeline_controller::*;
use super::super::model::*;

use flo_ui::*;
use flo_ui_files::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::collections::HashMap;

use serde_json;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
enum SubController {
    Canvas,
    Menu,
    Timeline,
    Toolbox
}

///
/// The editor controller manages the editing of a single file
///
pub struct EditorController {
    /// The main editor UI
    ui: Binding<Control>,

    /// The subcontrollers for this editor
    subcontrollers: HashMap<SubController, Arc<dyn Controller>>
}

impl EditorController {
    ///
    /// Creates a new editor controller from an animation
    /// 
    pub fn new<Anim: 'static+Animation+EditableAnimation>(animation: Anim) -> EditorController {
        let animation   = FloModel::new(animation);

        let canvas      = Arc::new(CanvasController::new(&animation));
        let menu        = Arc::new(MenuController::new(&animation));
        let timeline    = Arc::new(TimelineController::new(&animation));
        let toolbox     = Arc::new(ToolboxController::new(&animation));

        let ui          = bind(Self::ui());
        let mut subcontrollers: HashMap<SubController, Arc<dyn Controller>> = HashMap::new();

        subcontrollers.insert(SubController::Canvas,    canvas);
        subcontrollers.insert(SubController::Menu,      menu);
        subcontrollers.insert(SubController::Timeline,  timeline);
        subcontrollers.insert(SubController::Toolbox,   toolbox);

        EditorController {
            ui:             ui,
            subcontrollers: subcontrollers,
        }
    }

    ///
    /// Creates the menu bar control for this session
    ///
    fn menu_bar() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: End,
                y2: Offset(32.0)
            })
            .with_controller(&serde_json::to_string(&SubController::Menu).unwrap())
    }

    ///
    /// Creates the timeline control
    ///
    pub fn timeline() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: End,
                y2: Offset(256.0)
            })
            .with_controller(&serde_json::to_string(&SubController::Timeline).unwrap())
    }

    ///
    /// Creates the toolbar control
    ///
    pub fn toolbox() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: Offset(48.0),
                y2: End                    
            })
            .with_controller(&serde_json::to_string(&SubController::Toolbox).unwrap())
    }

    ///
    /// Creates the canvas control
    ///
    pub fn canvas() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: After,
                y1: Start,
                x2: Stretch(1.0),
                y2: End
            })
            .with_controller(&serde_json::to_string(&SubController::Canvas).unwrap())
    }

    ///
    /// Creates the UI tree for this controller
    ///
    pub fn ui() -> Control {
        use self::Position::*;

        let menu_bar    = Self::menu_bar();
        let timeline    = Self::timeline();
        let toolbar     = Self::toolbox();
        let canvas      = Self::canvas();

        Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                menu_bar,
                Control::container()
                    .with((vec![toolbar, canvas],
                        Bounds { x1: Start, y1: After, x2: End, y2: Stretch(1.0) })),
                timeline])
    }
}

impl Controller for EditorController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        let decoded_id = serde_json::from_str(id);

        if let Ok(decoded_id) = decoded_id {
            self.subcontrollers.get(&decoded_id).map(|controller_ref| controller_ref.clone())
        } else {
            None
        }
    }
}

/*
impl<Anim: FileAnimation+'static> FileController for EditorController<Anim> {
    /// The model that this controller needs to be constructed
    type Model = FloSharedModel<Anim>;

    ///
    /// Creates this controller with the specified instance model
    /// 
    fn open(model: FloSharedModel<Anim>) -> Self {
        unimplemented!()
    }
}
*/