use super::hsluv_picker_controller::*;

use ui::*;
use canvas::*;
use binding::*;

use std::sync::*;

///
/// Controller that makes it possible to pick a colour
/// 
pub struct ColorPickerController {
    ui:     BindRef<Control>,

    hsluv:  Arc<HsluvPickerController>
}

impl ColorPickerController {
    ///
    /// Creates a new color picker controller
    /// 
    pub fn new(color: &Binding<Color>) -> ColorPickerController {
        let ui      = Self::create_ui();
        let hsluv   = HsluvPickerController::new(color);

        ColorPickerController {
            ui:     ui,
            hsluv:  Arc::new(hsluv)
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn create_ui() -> BindRef<Control> {
        BindRef::from(computed(move || {
            Control::container()
                .with_controller("HSLUV")
                .with(Bounds::fill_all())
        }))
    }
}

impl Controller for ColorPickerController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
        match id {
            "HSLUV" => Some(self.hsluv.clone()),
            _       => None
        }
    }
}
