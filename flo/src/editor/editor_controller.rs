use super::menu_controller::*;
use super::canvas_controller::*;
use super::toolbox_controller::*;
use super::timeline_controller::*;
use super::keybinding_controller::*;
use super::controlbar_controller::*;

use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_ui_files::ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::marker::PhantomData;
use std::collections::HashMap;

use serde_json;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
enum SubController {
    Canvas,
    Menu,
    ControlBar,
    Timeline,
    Toolbox,
    KeyBindings
}

///
/// The editor controller manages the editing of a single file
///
pub struct EditorController<Anim: FileAnimation> {
    /// Phantom data so we can have the animation type
    anim: PhantomData<Anim>,

    /// The main editor UI
    ui: Binding<Control>,

    /// The subcontrollers for this editor
    subcontrollers: HashMap<SubController, Arc<dyn Controller>>
}

impl<Loader: 'static+FileAnimation> EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    ///
    /// Creates a new editor controller from an animation
    ///
    pub fn new(animation: Loader::NewAnimation) -> EditorController<Loader> {
        let animation   = FloModel::new(animation);

        Self::from_model(animation)
    }

    ///
    /// Creates a new editor controller from a model
    ///
    pub fn from_model(animation: FloModel<Loader::NewAnimation>) -> EditorController<Loader> {
        let canvas          = Arc::new(CanvasController::new(&animation));
        let menu            = Arc::new(MenuController::new(&animation));
        let timeline        = Arc::new(TimelineController::new(&animation));
        let toolbox         = Arc::new(ToolboxController::new(&animation));
        let control_bar     = Arc::new(ControlBarController::new(&animation));
        let key_bindings    = Arc::new(KeyBindingController::new());

        let ui              = bind(Self::ui());
        let mut subcontrollers: HashMap<SubController, Arc<dyn Controller>> = HashMap::new();

        subcontrollers.insert(SubController::Canvas,        canvas);
        subcontrollers.insert(SubController::Menu,          menu);
        subcontrollers.insert(SubController::Timeline,      timeline);
        subcontrollers.insert(SubController::Toolbox,       toolbox);
        subcontrollers.insert(SubController::ControlBar,    control_bar);
        subcontrollers.insert(SubController::KeyBindings,   key_bindings);

        EditorController {
            anim:           PhantomData,
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
                x2: Offset(TOOL_CONTROL_SIZE),
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
    /// Creates the control bar control
    ///
    pub fn control_bar() -> Control {
        Control::container()
            .with(Bounds::next_vert(28.0))
            .with(Appearance::Background(TIMESCALE_BACKGROUND))
            .with(Font::Size(12.0))
            .with(Font::Weight(FontWeight::Light))
            .with_controller(&serde_json::to_string(&SubController::ControlBar).unwrap())
    }

    ///
    /// Creates the key bindings control
    ///
    pub fn keybindings() -> Control {
        Control::container()
            .with(Bounds::next_vert(0.0))
            .with_controller(&serde_json::to_string(&SubController::KeyBindings).unwrap())
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
        let control_bar = Self::control_bar();
        let keybindings = Self::keybindings();

        Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                keybindings,
                menu_bar,
                Control::container()
                    .with((vec![toolbar, canvas],
                        Bounds { x1: Start, y1: After, x2: End, y2: Stretch(1.0) })),
                Control::empty()
                    .with(Bounds::next_vert(1.0))
                    .with(Appearance::Background(TIMESCALE_BORDER)),
                control_bar,
                Control::empty()
                    .with(Bounds::next_vert(1.0))
                    .with(Appearance::Background(TIMESCALE_LAYERS)),
                timeline])
    }
}

impl<Loader: 'static+FileAnimation> Controller for EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
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

impl<Loader: 'static+FileAnimation> FileController for EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    /// The model that this controller needs to be constructed
    type Model = FloSharedModel<Loader>;

    ///
    /// Creates this controller with the specified instance model
    ///
    fn open(model: FloModel<Loader::NewAnimation>) -> Self {
        Self::from_model(model)
    }
}

impl<Loader: 'static+FileAnimation> PartialEq for EditorController<Loader> {
    fn eq(&self, _rhs: &Self) -> bool {
        // These are never equal at the moment (this is needed for the binding library)
        false
    }
}
