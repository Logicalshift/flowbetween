use crate::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::sync::*;
use std::str::{FromStr};

#[derive(Clone, PartialEq, Eq, Hash, AsRefStr, Display, EnumString)]
enum SidebarAction {
    Unknown,

    /// The user has resized the sidebar
    Resize
}

///
/// Creates the user interface for the sidebar
///
fn sidebar_ui<Anim: 'static+EditableAnimation>(_model: &FloModel<Anim>) -> BindRef<Control> {
    use self::Position::*;

    let ui = bind(
        Control::container()
            .with(Bounds {
                x1: Start,
                y1: Start,
                x2: End,
                y2: End
            })
            .with((ActionTrigger::Resize, SidebarAction::Resize.as_ref()))
            .with(PointerBehaviour::ClickThrough)
            .with(vec![
                Control::empty()
                    .with(Bounds {
                        x1: Start,
                        y1: Start,
                        x2: End,
                        y2: Offset(30.0)
                    })
            ])
        );

    BindRef::from(ui)
}

///
/// Creates the sidebar controller
///
pub fn sidebar_controller<Anim: 'static+EditableAnimation>(model: &FloModel<Anim>) -> impl Controller {
    // TODO: Create the set of subcontrollers

    // Parameters used for configuring the sidebar
    let height      = bind(0.0);

    // Keep a copy of the model for the runtime
    let model       = Arc::new(model.clone());

    ImmediateController::empty(move |events, actions, resources| {
        // Start by taking the model from the main controller
        let model       = model.clone();
        let height      = height.clone();

        let mut actions = actions;
        let mut events  = events;

        async move {
            // TODO: Set up the subcontrollers

            // Set up the UI
            let ui  = sidebar_ui(&model);
            actions.send(ControllerAction::SetUi(ui.clone())).await.ok();

            // Process events
            while let Some(next_event) = events.next().await {
                match next_event {
                    ControllerEvent::Action(action_name, param) => {
                        let action_name: &str   = &action_name;
                        let action              = SidebarAction::from_str(action_name).unwrap_or(SidebarAction::Unknown);

                        // Decode the action
                        match (action, param) {
                            (SidebarAction::Resize, ActionParameter::Size(_new_width, new_height)) => {
                                // The size is used to determine which sidebar items are displayed as 'open'
                                height.set(new_height);
                            }

                            _ => { /* Unrecognised action */ }
                        }
                    }

                    _ => { }
                }
            }
        }
    })
}
