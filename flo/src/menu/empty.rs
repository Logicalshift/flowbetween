use ui::*;
use binding::*;

use std::sync::*;

///
/// Controller used when no other menu controller is available
/// 
pub struct EmptyMenuController {
    ui:         BindRef<Control>,
    view_model: Arc<NullViewModel>
}

impl EmptyMenuController {
    pub fn new() -> EmptyMenuController {
        EmptyMenuController { 
            ui:         BindRef::from(bind(Control::empty())),
            view_model: Arc::new(NullViewModel::new())
        }
    }
}

impl Controller for EmptyMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
