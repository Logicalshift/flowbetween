use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_animation::*;

use std::sync::*;

///
/// Creates the document settings controller and UI
///
/// This is used to set things like the size of the document and the frame rate
///
pub fn document_settings_controller<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Controller {
    ImmediateController::empty(|events, _actions, _resources| {
        async move {
            // Set up the viewmodel bindings

            // Set up the UI

            // Run the events
            let mut events = events;
            while let Some(event) = events.next().await {

            }
        }
    })
}
