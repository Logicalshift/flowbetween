use super::timeline_controller::*;

use flo_ui::*;
use flo_binding::*;

///
/// Controller class that displays and edits the layer names and allows adding new layers to the drawing
/// 
pub struct TimelineLayerController {

}

impl TimelineLayerController {

}

impl Controller for TimelineLayerController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()))
    }
}