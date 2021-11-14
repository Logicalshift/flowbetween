use crate::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Creates the sidebar controller
///
pub fn sidebar_controller<Anim: 'static+EditableAnimation>(model: &FloModel<Anim>) -> impl Controller {
    let model = model.clone();

    ImmediateController::new(ControllerResources::new(), BindRef::from(bind(Control::empty())), 
        move |_events, _actions, _resources| {
            // Start by taking the model from the main controller
            let model = model.clone();

            async move {

            }
        })
}
