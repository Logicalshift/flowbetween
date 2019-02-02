use super::event::*;
use super::action::*;
use super::app_state::*;
use super::regulator::*;

use flo_ui::*;
use flo_ui::session::*;

use futures::*;

use std::sync::*;

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<Ui, Cocoa>(ui: &Ui, cocoa: &Cocoa) -> impl Future<Item=()>
where   Ui:     UserInterface<Vec<UiEvent>, Vec<UiUpdate>, ()>,
        Cocoa:  UserInterface<Vec<AppAction>, Vec<AppEvent>, ()> {
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

    // Pipe the updates into the cocoa side
    let update_state = state.clone();
    let handle_updates = ui_stream
        .map(move |updates| {
            updates.into_iter()
                .flat_map(|update| update_state.lock().unwrap().map_update(update))
                .collect::<Vec<_>>()
        })
        .forward(cocoa_sink)
        .map(|_| ());

    // Pipe the events the other way
    let event_state     = state;
    let handle_events   = cocoa_events
        .map(move |events| {
            events.into_iter()
                .flat_map(|event| event_state.lock().unwrap().map_event(event))
                .collect::<Vec<_>>()
        })
        .forward(ui_events)
        .map(|_| ());

    // Updates stop when any of the streams end
    handle_updates.select(handle_events)
        .map(|_| ())
}
