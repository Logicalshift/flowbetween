use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_stream::*;
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
                            .with(ControlAttribute::Padding((4, 2), (0, 2)))
                            .with(Hover::Tooltip("Undo".to_string()))
                            .with(State::Enabled(Property::bound("CanUndo")))
                            .with((ActionTrigger::Command(Command::with_id("undo").named("Undo")), "Undo"))
                            .with((ActionTrigger::Click, "Undo")),
                        Control::button()
                            .with(redo.clone())
                            .with(Bounds::next_horiz(32.0))
                            .with(ControlAttribute::Padding((0, 2), (4, 2)))
                            .with(Hover::Tooltip("Redo".to_string()))
                            .with(State::Enabled(Property::bound("CanRedo")))
                            .with((ActionTrigger::Command(Command::with_id("redo").named("Redo")), "Redo"))
                            .with((ActionTrigger::Click, "Redo")),
                    ]),

                Control::empty()
                    .with(Bounds::next_horiz(12.0)),
            ])
    }).into()
}

///
/// Carries out the undo operation on the animation
///
async fn perform_undo<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<UndoableAnimation<Anim>>>) {
    // We use the 'prepare to undo' event to signal anything that's listening that an undo is in progress for the following set of edits
    model.edit().publish(Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo("Undo_START".into()))])).await;

    if let Err(undo_err) = model.undo().await {
        warn!("Undo failed: {:?}", undo_err);
    }

    model.edit().publish(Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo("Undo_FINISH".into()))])).await;
}

///
/// Carries out the redo operation on the animation
///
async fn perform_redo<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<UndoableAnimation<Anim>>>) {
    // We use the 'prepare to undo' event to signal anything that's listening that an redo is in progress for the following set of edits
    model.edit().publish(Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo("Redo_START".into()))])).await;

    if let Err(redo_err) = model.redo().await {
        warn!("Redo failed: {:?}", redo_err);
    }

    model.edit().publish(Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo("Redo_FINISH".into()))])).await;
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
            let undo        = resources.images().register_named("undo", svg_static(include_bytes!("../../svg/menu_controls/undo.svg")));
            let redo        = resources.images().register_named("redo", svg_static(include_bytes!("../../svg/menu_controls/redo.svg")));

            // Properties
            let can_undo    = bind_stream(model.follow_undo_log_size_changes(), PropertyValue::Bool(false), |_, size| PropertyValue::Bool(size.undo_depth > 0));
            let can_redo    = bind_stream(model.follow_undo_log_size_changes(), PropertyValue::Bool(false), |_, size| PropertyValue::Bool(size.redo_depth > 0));
            actions.send(ControllerAction::SetPropertyBinding("CanUndo".into(), can_undo.clone().into())).await.ok();
            actions.send(ControllerAction::SetPropertyBinding("CanRedo".into(), can_redo.clone().into())).await.ok();

            // Set up the UI
            let ui          = edit_bar_ui(&model, undo, redo);
            actions.send(ControllerAction::SetUi(ui)).await.ok();

            // Receive events for this controller
            while let Some(next_event) = events.next().await {
                // Dispatch each event as it arrives
                match next_event {
                    ControllerEvent::Action(action_name, _params) => {
                        match action_name.as_str() {
                            "Undo"  => { if can_undo.get() == PropertyValue::Bool(true) { perform_undo(&model).await; } }
                            "Redo"  => { if can_redo.get() == PropertyValue::Bool(true) { perform_redo(&model).await; } }

                            _       => { }
                        }
                    }
                }
            }
        }
    })
}
