use super::super::style::*;

use ui::*;
use canvas::*;
use binding::*;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    ui:         Binding<Control>
}

impl TimelineController {
    pub fn new() -> TimelineController {
        let ui = bind(Control::scrolling_container()
            .with(Bounds::fill_all())
            .with(Scroll::ContentSize(6000.0, 256.0))
            .with(Scroll::AllowScroll(true, true))
            .with(Scroll::AutoHide(false, true))
            .with(Appearance::Background(TIMELINE_BACKGROUND))
            .with(vec![
                Control::empty()
                    .with(Appearance::Background(Color::Rgba(0.4, 0.0, 0.0, 1.0)))
                    .with(Bounds::fill_all())
            ]));

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
