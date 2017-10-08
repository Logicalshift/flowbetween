use ui::*;

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController {
    ui: Binding<Control>
}

impl CanvasController {
    pub fn new() -> CanvasController {
        let ui = bind(Control::empty());

        CanvasController {
            ui: ui
        }
    }
}

impl Controller for CanvasController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }
}
