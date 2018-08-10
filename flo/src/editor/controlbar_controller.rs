use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// The control bar provides a home for the basic animation editing and playback controls
/// 
pub struct ControlBarController {
    ui: BindRef<Control>
}

impl ControlBarController {
    ///
    /// Creates a new control bar controller
    /// 
    pub fn new<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>) -> ControlBarController {
        let ui = Self::ui();

        ControlBarController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui() -> BindRef<Control> {
        let ui = bind(Control::empty());

        BindRef::from(ui)
    }
}

impl Controller for ControlBarController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}