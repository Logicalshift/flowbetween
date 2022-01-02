use super::panel::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_rope::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream;

use std::mem;
use std::sync::*;
use std::str::{FromStr};
use std::collections::{HashSet, HashMap};

/// Size of the title bar for a panel
const TITLE_SIZE: f64   = 20.0;

/// Size of the gap between panels
const GAP: f64          = 3.0;

#[derive(Clone, PartialEq, Eq, Hash, AsRefStr, Display, EnumString)]
enum SidebarAction {
    Unknown,

    /// The user has resized the sidebar
    Resize,

    /// The user has requested a panel to be opened
    OpenPanel(String),

    /// The user has requested a panel to be closed
    ClosePanel(String)
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
}

///
/// Creates the user interface for a set of sidebar panels
///
fn sidebar_ui(panels: Vec<&SidebarPanel>, open_panels: BindRef<Vec<String>>, total_height: BindRef<f64>) -> BindRef<Control> {
    use self::Position::*;

    // Collect the data for the panels
    let panel_heights   = panels.iter()
        .map(|panel| (panel.identifier().to_string(), panel.height().clone()))
        .collect::<HashMap<_, _>>();
    let panels          = panels.iter()
        .map(|panel| (panel.identifier().to_string(), panel.title().to_string(), panel.height().clone(), panel.active().clone()))
        .collect::<Vec<_>>();

    // Create the UI for this set of panels
    let ui = computed(move || {
        let total_height        = total_height.get();
        let open_panel_order    = open_panels.get();

        // Sidebars are opened in order until we run out of space
        let num_panels          = panels.len() as f64;
        let mut used_height     = TITLE_SIZE * num_panels + GAP * num_panels;
        let mut open_panels     = HashSet::new();

        for open_panel_id in open_panel_order.iter() {
            if let Some(panel_height) = panel_heights.get(open_panel_id) {
                // The panel will use its requested height (the size of the title and gap between panels is already in used_height)
                let panel_height = panel_height.get();

                // Open if there's enough room
                if used_height + panel_height < total_height {
                    open_panels.insert(open_panel_id.clone());
                    used_height += panel_height;
                }
            } 
        }

        // Create the UI for each of the panels
        let mut panel_controls  = vec![];
        let mut last_position   = Start;
        for (panel_id, panel_title, panel_height, panel_active) in panels.iter() {
            // Fetch the height and weather or not it's open
            let is_active           = panel_active.get();
            let is_open             = open_panels.contains(panel_id);

            // Title control
            let title_action        = if is_open { format!("ClosePanel {}", panel_id) } else { format!("OpenPanel {}", panel_id) };
            let title_control       = Control::container()
                .with(Bounds { x1: Start, y1: last_position, x2: End, y2: Offset(TITLE_SIZE as _)} )
                .with(Appearance::Background(SIDEBAR_TITLE_BACKGROUND))
                .with((ActionTrigger::Click, title_action))
                .with(vec![
                    Control::empty().with(Bounds { x1: Start, y1: Start, x2: Offset(28.0), y2: End }).with(PointerBehaviour::ClickThrough),
                    if is_active {
                        Control::label().with(Font::Weight(FontWeight::ExtraBold)).with(Appearance::Foreground(DEFAULT_TEXT))
                    } else {
                        Control::label().with(Font::Weight(FontWeight::Light)).with(Appearance::Foreground(DIM_TEXT))
                    }
                        .with(Font::Size(12.0))
                        .with(Bounds { x1: After, y1: Start, x2: Stretch(1.0), y2: End })
                        .with(panel_title)
                        .with(PointerBehaviour::ClickThrough),
                    Control::empty().with(Bounds { x1: After, y1: Start, x2: Offset(28.0), y2: End }).with(PointerBehaviour::ClickThrough),
                ]);

            // Panel content
            let content_controls    = if is_open {
                // Panel is open
                let panel_height = panel_height.get();
                vec![
                    Control::container().with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(panel_height as _) })
                        .with(Font::Size(11.0))
                        .with(Font::Weight(FontWeight::Light))
                        .with((ActionTrigger::Click, "BackgroundClick"))
                        .with_controller(panel_id)
                ]
            } else {
                // Panel is closed
                vec![]
            };

            // Following gap
            let following_gap       = Control::empty().with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(GAP as _)} );

            // Add to the controls
            panel_controls.push(title_control);
            panel_controls.extend(content_controls);
            panel_controls.push(following_gap);

            // Next panel is positioned after this one
            last_position = After;
        }

        // Final UI
        Control::container()
            .with(Bounds {
                x1: Start,
                y1: Start,
                x2: End,
                y2: End
            })
            .with((ActionTrigger::Click, "BackgroundClick"))
            .with((ActionTrigger::Resize, SidebarAction::Resize.as_ref()))
            .with(panel_controls)
    });

    BindRef::from(ui)
}

///
/// Parses an OpenPanel or ClosePanel action name (returning SidebarAction::Unknown if the action can't be parsed)
///
fn parse_open_close_action(action_name: &str) -> SidebarAction {
    if action_name.starts_with("OpenPanel ") {
        SidebarAction::OpenPanel(action_name["OpenPanel ".len()..].to_string())
    } else if action_name.starts_with("ClosePanel ") {
        SidebarAction::ClosePanel(action_name["ClosePanel ".len()..].to_string())
    } else {
        SidebarAction::Unknown
    }
}

///
/// Converts a stream of controller events to sidebar events
///
fn sidebar_events<ControllerEvents: Stream<Item=ControllerEvent>>(events: ControllerEvents) -> impl Stream<Item=SidebarEvent> {
    events.map(|controller_event| {
        match controller_event {
            ControllerEvent::Action(name, param)    => SidebarEvent::Action(name, param),
        }
    })
}

///
/// Generates events for when the list of panels change
///
/// These register or deregister controllers and update the UI accordingly
///
fn panel_change_events<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, total_height: BindRef<f64>) -> impl Stream<Item=SidebarEvent> {
    let open_panels     = BindRef::from(model.sidebar().open_sidebars.clone());

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
            let panels = new_panels.read_cells(0..new_panels.len()).collect();
            let new_ui = sidebar_ui(panels, open_panels.clone(), total_height.clone());
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
        let panel_events    = panel_change_events(&model, BindRef::from(height.clone())).boxed();

        let mut events      = stream::select_all(vec![events, panel_events]);

        async move {
            // Process events
            while let Some(next_event) = events.next().await {
                match next_event {
                    SidebarEvent::AddController(name, controller)   => { actions.send(ControllerAction::AddSubController(name, controller)).await.ok(); }
                    SidebarEvent::RemoveController(name)            => { actions.send(ControllerAction::RemoveSubController(name)).await.ok(); }
                    SidebarEvent::SetUi(new_ui)                     => { actions.send(ControllerAction::SetUi(new_ui)).await.ok(); }

                    SidebarEvent::Action(action_name, param) => {
                        let action_name: &str   = &action_name;
                        let action              = SidebarAction::from_str(action_name)
                            .unwrap_or_else(|_| parse_open_close_action(action_name));

                        // Decode the action
                        match (action, param) {
                            (SidebarAction::Resize, ActionParameter::Size(_new_width, new_height)) => {
                                // The size is used to determine which sidebar items are displayed as 'open'
                                height.set(new_height as f64);
                            }

                            (SidebarAction::OpenPanel(panel_id), _) => {
                                // Add the panel to the start of the open list
                                let mut open_panels = model.sidebar().open_sidebars.get();

                                open_panels.retain(|existing_id| existing_id != &panel_id);
                                open_panels.insert(0, panel_id);

                                model.sidebar().open_sidebars.set(open_panels);
                            }

                            (SidebarAction::ClosePanel(panel_id), _) => {
                                // Remove the panel from the open list
                                let mut open_panels = model.sidebar().open_sidebars.get();

                                open_panels.retain(|existing_id| existing_id != &panel_id);

                                model.sidebar().open_sidebars.set(open_panels);
                            }

                            _ => { /* Unrecognised action */ }
                        }
                    }
                }
            }
        }
    })
}
