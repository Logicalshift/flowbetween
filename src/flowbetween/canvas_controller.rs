use ui::*;

use std::sync::*;

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController {
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>
}

impl CanvasController {
    pub fn new() -> CanvasController {
        let ui = bind(Control::empty());

        CanvasController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         ui
        }
    }
}

impl Controller for CanvasController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
