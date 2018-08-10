use super::keyframe_controls_controller::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// The control bar provides a home for the basic animation editing and playback controls
/// 
pub struct ControlBarController {
    /// The UI for this control bar
    ui: BindRef<Control>,

    /// The keyframe controls controller
    keyframe_controls: Arc<KeyFrameControlsController>
}

impl ControlBarController {
    ///
    /// Creates a new control bar controller
    /// 
    pub fn new<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>) -> ControlBarController {
        // Create the UI
        let ui                  = Self::ui();

        // Create the subcontrollers
        let keyframe_controls   = KeyFrameControlsController::new(model);
        let keyframe_controls   = Arc::new(keyframe_controls);

        // Build the controller itself
        ControlBarController {
            ui:                 ui,
            keyframe_controls:  keyframe_controls
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui() -> BindRef<Control> {
        // Create the UI itself
        let ui = Control::container()
            .with(Bounds::fill_all())
            .with(ControlAttribute::Padding((0, 2), (0, 2)))
            .with(vec![
                Control::empty()
                    .with(Bounds::stretch_horiz(1.0)),
                Control::container()
                    .with_controller("KeyFrameControls")
                    .with(Bounds::next_horiz(166.0)),
                Control::empty()
                    .with(Bounds::next_horiz(32.0))
            ]);

        // Create the binding
        let ui = bind(ui);
        BindRef::from(ui)
    }
}

impl Controller for ControlBarController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        match id {
            "KeyFrameControls"  => Some(self.keyframe_controls.clone()),

            _                   => None
        }
    }
}
