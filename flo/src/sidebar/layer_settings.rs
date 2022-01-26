use super::panel_style::*;
use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// Creates the UI for editing the settings for the currently selected layer
///
pub fn layer_settings_controller<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>, height: Binding<f64>) -> impl Controller {
    ImmediateController::empty(move |events, actions, _resources| {
        async move {

        }
    })
}
