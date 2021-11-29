use crate::model::*;

use flo_ui::*;
use flo_rope::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream;

use std::mem;
use std::sync::*;
use std::str::{FromStr};
use std::collections::{HashSet};

#[derive(Clone, PartialEq, Eq, Hash, AsRefStr, Display, EnumString)]
enum SidebarAction {
    Unknown,

    /// The user has resized the sidebar
    Resize
}

///
/// An event for the sidebar (we convert from controller events so we can mix in events from other sources, such as dealing with panel changes)
///
enum SidebarEvent {
    /// Add a subcontroller with the specified name to the sidebar
    AddController(String, Arc<dyn Controller>),

    /// The specified controller is no longer being managed as part of the sidebar
    RemoveController(String),

    /// Update the sidebar UI to the specified binding (when the list of panels change we re-bind the UI to avoid a potential race between updating the controllers and updating the UI)
    SetUi(BindRef<Control>),

    /// An action was generated from the UI
    Action(String, ActionParameter),

    /// Tick events are generated every time an update is completed
    Tick,
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
/// Converts a stream of controller events to sidebar events
///
fn sidebar_events<ControllerEvents: Stream<Item=ControllerEvent>>(events: ControllerEvents) -> impl Stream<Item=SidebarEvent> {
    events.map(|controller_event| {
        match controller_event {
            ControllerEvent::Action(name, param)    => SidebarEvent::Action(name, param),
            ControllerEvent::Tick                   => SidebarEvent::Tick
        }
    })
}

///
/// Generates events for when the list of panels change
///
/// These register or deregister controllers and update the UI accordingly
///
fn panel_change_events<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=SidebarEvent> {
    let model           = Arc::clone(model);

    // Follow the list of panels defined for the sidebar
    let panel_changes   = follow(model.sidebar().panels.clone());

    // Keep a list of the panel IDs we know about (these correspond to the new controllers we know about)
    let known_panels    = Arc::new(Mutex::new(HashSet::<String>::new()));

    // Run the panel changes stream
    panel_changes
        .map(move |new_panels| {
            // Build the list of events for these changes
            let mut events              = vec![];

            // Work out which controllers were added and which were removed
            let mut known_panels        = known_panels.lock().unwrap();
            let mut new_known_panels    = new_panels.read_cells(0..new_panels.len()).map(|panel| panel.identifier().to_string()).collect::<HashSet<_>>();

            let new_panel_ids           = new_known_panels.iter().filter(|id| !known_panels.contains(*id)).cloned().collect::<HashSet<_>>();
            let deleted_panel_ids       = known_panels.iter().filter(|id| !new_known_panels.contains(*id)).cloned().collect::<Vec<_>>();

            // Send the list of new controllers
            for panel in new_panels.read_cells(0..new_panels.len()) {
                if new_panel_ids.contains(panel.identifier()) {
                    events.push(SidebarEvent::AddController(panel.identifier().to_string(), panel.controller()));
                }
            }

            mem::swap(&mut *known_panels, &mut new_known_panels);

            // Update the UI to use the new controllers
            let new_ui = sidebar_ui(&*model);
            events.push(SidebarEvent::SetUi(new_ui));

            // Send the list of deleted controllers
            events.extend(deleted_panel_ids.into_iter().map(|panel_id| SidebarEvent::RemoveController(panel_id)));

            events
        })
        .map(|changes| stream::iter(changes))
        .flatten()
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

    ImmediateController::empty(move |events, actions, _resources| {
        // Start by taking the model from the main controller
        let model           = model.clone();
        let height          = height.clone();

        let mut actions     = actions;
        let events          = sidebar_events(events).boxed();
        let panel_events    = panel_change_events(&model).boxed();

        let mut events      = stream::select_all(vec![events, panel_events]);

        async move {
            // TODO: Set up the subcontrollers

            // Set up the UI
            let ui  = sidebar_ui(&model);
            actions.send(ControllerAction::SetUi(ui.clone())).await.ok();

            // Process events
            while let Some(next_event) = events.next().await {
                match next_event {
                    SidebarEvent::Action(action_name, param) => {
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
