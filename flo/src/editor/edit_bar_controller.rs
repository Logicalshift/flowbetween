use crate::model::*;

use flo_ui::*;
use flo_animation::*;
use flo_animation::undo::*;

use futures::prelude::*;

///
/// The edit bar controller provides some standard editing controls, starting with undo
///
pub fn edit_bar_controller<Anim: 'static+EditableAnimation>(model: &FloModel<UndoableAnimation<Anim>>) -> impl Controller {
    ImmediateController::empty(move |events, actions, _resources| {
        async move {
            // Receive events for this controller
            let mut events = events;

            while let Some(next_event) = events.next().await {
            }
        }
    })
}
