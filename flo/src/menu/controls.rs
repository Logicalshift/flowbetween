use super::super::style::*;

use flo_ui::*;

///
/// A divider for the menu controls
///
pub fn divider() -> Control {
    Control::container()
        .with(vec![
            Control::empty()
                .with(Bounds::next_horiz(5.0)),
            Control::empty()
                .with(Bounds::next_horiz(2.0))
                .with(Appearance::Background(MENU_BACKGROUND_ALT)),
            Control::empty()
                .with(Bounds::next_horiz(5.0)),
        ])
        .with(Bounds::next_horiz(12.0))
}
