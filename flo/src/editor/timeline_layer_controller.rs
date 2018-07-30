use super::timeline_controller::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

///
/// Controller class that displays and edits the layer names and allows adding new layers to the drawing
/// 
pub struct TimelineLayerController {

}

impl TimelineLayerController {
    ///
    /// Creates a new timeline layer controller
    /// 
    pub fn new<Anim: 'static+Animation>(_model: &FloModel<Anim>) -> TimelineLayerController {
        TimelineLayerController {

        }
    }
}

impl Controller for TimelineLayerController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()
            .with(Bounds::fill_all())
            .with(Appearance::Background(Color::Rgba(1.0, 0.0, 0.0, 1.0)))))
    }
}