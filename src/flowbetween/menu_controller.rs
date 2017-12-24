use super::style::*;

use ui::*;
use binding::*;

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
        let ui = bind(Control::empty()
            .with(Bounds::fill_all())
            .with(vec![
                Control::empty()
                    .with(Bounds::next_horiz(6.0))
                    .with(ControlAttribute::Background(MENU_BACKGROUND_ALT)),
                Control::empty()
                    .with(Bounds::next_horiz(2.0)),
                Control::label()
                    .with("FlowBetween")
                    .with(ControlAttribute::FontSize(17.0))
                    .with(Bounds::next_horiz(160.0))
            ])
            .with(ControlAttribute::Background(MENU_BACKGROUND)));

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
