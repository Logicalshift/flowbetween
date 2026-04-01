use super::event::*;
use super::action::*;
use super::app_state::*;
use super::regulator::*;

use flo_ui::*;
use flo_ui::session::*;
use flo_stream::*;

use futures::*;

use std::sync::*;
use std::collections::HashMap;

///
/// Indicates when a tick event is pending and/or if we've suspended updates
///
#[derive(Copy, Clone, PartialEq)]
enum TickState {
    NoTick,
    RequestedTick,
    Suspended
}

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<Ui, Cocoa>(ui: &Ui, cocoa: &Cocoa) -> impl Future<Output=()>
where   Ui:                     UserInterface<Vec<UiEvent>, Vec<UiUpdate>, ()>,
        Cocoa:                  UserInterface<Vec<AppAction>, Vec<AppEvent>, ()>,
        Ui::UpdateStream:       Unpin,
        Cocoa::UpdateStream:    Unpin {
    // The state keeps track of how the events from the UI side map to the Cocoa side
    let state           = Arc::new(Mutex::new(AppState::new()));

    // Create the stream for updates coming from the UI side
    let ui_stream       = ui.get_updates();
    let cocoa_sink      = cocoa.get_input_sink();

    // ... and the stream for sending the events the other way
    let ui_events       = ui.get_input_sink();
    let cocoa_events    = cocoa.get_updates();

    // Group the events from the UI stream into as small batches as possible
    let ui_stream       = group_stream(ui_stream);
    let cocoa_events    = group_stream(cocoa_events);

    // Tracks if we've requested a tick or not
    let requested_tick  = Arc::new(Mutex::new(TickState::NoTick));

    // Pipe the updates into the cocoa side
    let update_tick     = requested_tick.clone();
    let update_state    = state.clone();
    let handle_updates  = ui_stream
        .map(move |updates| {
            updates.map(|updates| {
                // If some updates arrive and we're not waiting for a tick, start waiting for a tick
                let mut request_tick    = vec![];
                let mut update_tick     = update_tick.lock().unwrap();

                if *update_tick == TickState::NoTick && updates.len() > 0 {
                    request_tick.push(AppAction::Window(0, WindowAction::RequestTick));
                    *update_tick = TickState::RequestedTick;
                }

                // Follow the tick request by the set of updates
                request_tick.into_iter().chain(
                    updates.into_iter()
                        .flat_map(|update| update_state.lock().unwrap().map_update(update)))
                    .collect::<Vec<_>>()
            })
        })
        .forward(cocoa_sink.to_sink())
        .map(|_| ());

    // Pipe the events the other way
    let event_tick      = requested_tick.clone();
    let event_state     = state;
    let handle_events   = cocoa_events
        .map(move |events| {
            events.map(|events| {
                // If an event arrives while we're waiting for a tick, suspend updates until at least one tick arrives
                let mut suspend_updates = vec![];
                let mut event_tick      = event_tick.lock().unwrap();

                if *event_tick == TickState::RequestedTick {
                    suspend_updates.push(UiEvent::SuspendUpdates);
                    *event_tick = TickState::Suspended;
                }

                let updates = suspend_updates.into_iter().chain(
                    events.into_iter()
                        .flat_map(|event| event_state.lock().unwrap().map_event(event)))
                    .collect::<Vec<_>>();

                // Combine any painting events for a particular view
                let mut combined_painting   = vec![];
                let mut painting_for_view   = HashMap::new();

                for update in updates.into_iter() {
                    match update {
                        UiEvent::Action(controller_path, action_name, ActionParameter::Paint(device, painting)) => {
                            // Store all the painting actions together
                            let actions = painting_for_view.entry((controller_path, action_name, device)).or_insert_with(|| vec![]);
                            actions.extend(painting);
                        },

                        other_action => {
                            // Other actions are just pushed straight away
                            combined_painting.push(other_action)
                        }
                    }
                }

                // Add in the combined painting actions
                for ((controller_path, action_name, device), paint_actions) in painting_for_view.into_iter() {
                    combined_painting.push(UiEvent::Action(controller_path, action_name, ActionParameter::Paint(device, paint_actions)));
                }

                let mut updates = combined_painting;

                // If updates are suspended and a tick has occurred, then resume them after the events have been sent
                if updates.contains(&UiEvent::Tick) {
                    if *event_tick == TickState::Suspended {
                        updates.push(UiEvent::ResumeUpdates);
                    }
                    *event_tick = TickState::NoTick;
                } else {
                    updates.push(UiEvent::Tick);
                }

                updates
            })
        })
        .forward(ui_events.to_sink())
        .map(|_| ());

    // Updates stop when any of the streams end
    future::select(handle_updates, handle_events)
        .map(|_| ())
}
