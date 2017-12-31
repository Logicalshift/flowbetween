use ui::*;
use binding::*;

use std::sync::*;

pub const INKMENUCONTROLLER: &str = "InkMenu";

///
/// Controller used for the ink tool
/// 
pub struct InkMenuController {
    ui:         BindRef<Control>,
    view_model: Arc<NullViewModel>
}

impl InkMenuController {
    ///
    /// Creates a new ink menu controller
    /// 
    pub fn new() -> InkMenuController {
        InkMenuController { 
            ui:         BindRef::from(bind(Control::label()
                .with("Ink menu will go here")
                .with(Bounds::fill_all()))),
            view_model: Arc::new(NullViewModel::new())
        }
    }
}

impl Controller for InkMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
