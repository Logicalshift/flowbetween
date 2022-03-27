use crate::model::*;
use crate::style::*;

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
        Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                Control::empty()
                    .with(Bounds::next_horiz(1.0))
                    .with(Appearance::Background(MENU_BACKGROUND_ALT)),
                Control::empty()
                    .with(Bounds::stretch_horiz(1.0)),

                Control::container()
                    .with(Hint::Class("button-group".to_string()))
                    .with(Bounds::next_horiz(64.0))
                    .with(ControlAttribute::Padding((0, 6), (0, 6)))
                    .with(vec![
                        Control::button()
                            .with(undo.clone())
                            .with(Bounds::next_horiz(32.0))
                            .with((ActionTrigger::Click, "Undo")),
                        Control::button()
                            .with(redo.clone())
                            .with(Bounds::next_horiz(32.0))
                            .with((ActionTrigger::Click, "Redo")),
                    ]),

                Control::empty()
                    .with(Bounds::next_horiz(12.0)),
            ])
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
            let undo    = resources.images().register_named("undo", svg_static(include_bytes!("../../svg/menu_controls/undo.svg")));
            let redo    = resources.images().register_named("redo", svg_static(include_bytes!("../../svg/menu_controls/redo.svg")));

            // Set up the UI
            let ui      = edit_bar_ui(&model, undo, redo);
            actions.send(ControllerAction::SetUi(ui)).await.ok();

            // Receive events for this controller
            while let Some(next_event) = events.next().await {
            }
        }
    })
}
