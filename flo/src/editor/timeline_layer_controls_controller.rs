use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Controller that provides controls for adding/deleting/editing layers (generally displayed above the main layer list)
/// 
pub struct TimelineLayerControlsController {

}

impl TimelineLayerControlsController {
    ///
    /// Creates a new timeline layer controls controller
    /// 
    pub fn new<Anim: 'static+Animation>(model: &FloModel<Anim>) -> TimelineLayerControlsController {
        TimelineLayerControlsController {

        }
    }
}

impl Controller for TimelineLayerControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                Control::empty()
                    .with(Bounds::stretch_vert(1.0)),
                Control::empty()
                    .with(Appearance::Background(TIMESCALE_BORDER))
                    .with(Bounds::next_vert(1.0))
            ])))
    }
}