use super::super::style::*;

use ui::*;
use binding::*;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    ui:         Binding<Control>
}

impl TimelineController {
    pub fn new() -> TimelineController {
        let ui = bind(Control::empty()
            .with(Bounds::fill_all())
            .with(Appearance::Background(TIMELINE_BACKGROUND)));

        TimelineController {
            ui:         ui
        }
    }
}

impl Controller for TimelineController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }
}
