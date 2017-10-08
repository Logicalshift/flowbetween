use ui::*;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    ui: Binding<Control>
}

impl TimelineController {
    pub fn new() -> TimelineController {
        let ui = bind(Control::empty());

        TimelineController {
            ui: ui
        }
    }
}

impl Controller for TimelineController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }
}
