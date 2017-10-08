use ui::*;

///
/// The menu controller handles the menbu at the top of the UI
///
pub struct MenuController {
    ui: Binding<Control>
}

impl MenuController {
    pub fn new() -> MenuController {
        let ui = bind(Control::empty());

        MenuController {
            ui: ui
        }
    }
}

impl Controller for MenuController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }
}
