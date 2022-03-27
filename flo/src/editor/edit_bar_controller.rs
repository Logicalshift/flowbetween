use crate::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::undo::*;

use futures::prelude::*;

use std::sync::*;

///
/// Creates the UI binding for the edit controller
///
fn edit_bar_ui<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<UndoableAnimation<Anim>>>, undo: Resource<Image>, redo: Resource<Image>) -> BindRef<Control> {
    computed(move || {
        Control::empty()
    }).into()
}

///
/// The edit bar controller provides some standard editing controls, starting with undo
///
pub fn edit_bar_controller<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<UndoableAnimation<Anim>>>) -> impl Controller {
    let model = model.clone();

    ImmediateController::empty(move |events, actions, resources| {
        let model = model.clone();

        async move {
            let mut events  = events;
            let mut actions = actions;

            // Load resources
            let undo    = resources.images().register(svg_static(include_bytes!("../../svg/menu_controls/undo.svg")));
            let redo    = resources.images().register(svg_static(include_bytes!("../../svg/menu_controls/redo.svg")));

            resources.images().assign_name(&undo, "undo");
            resources.images().assign_name(&redo, "redo");

            // Set up the UI
            let ui      = edit_bar_ui(&model, undo, redo);
            actions.send(ControllerAction::SetUi(ui)).await.ok();

            // Receive events for this controller
            while let Some(next_event) = events.next().await {
            }
        }
    })
}
