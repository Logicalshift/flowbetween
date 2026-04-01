use flo_ui::*;
use flo_binding::*;

///
/// Controller used when no other menu controller is available
///
pub struct EmptyMenuController {
    ui:         BindRef<Control>
}

impl EmptyMenuController {
    pub fn new() -> EmptyMenuController {
        EmptyMenuController {
            ui:         BindRef::from(bind(Control::empty()))
        }
    }
}

impl Controller for EmptyMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}
