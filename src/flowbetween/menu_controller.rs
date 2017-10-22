use ui::*;

use std::sync::*;

///
/// The menu controller handles the menbu at the top of the UI
///
pub struct MenuController {
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>
}

impl MenuController {
    pub fn new() -> MenuController {
        let ui = bind(Control::empty());

        MenuController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         ui
        }
    }
}

impl Controller for MenuController {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
