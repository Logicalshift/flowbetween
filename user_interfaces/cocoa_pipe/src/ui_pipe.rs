use super::event::*;
use super::action::*;
use super::app_state::*;

use flo_ui::*;
use flo_ui::session::*;

use futures::*;

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<Ui, Cocoa>(ui: &Ui, cocoa: &Cocoa) -> impl Future<Item=()>
where   Ui:     UserInterface<Vec<UiEvent>, Vec<UiUpdate>, ()>,
        Cocoa:  UserInterface<Vec<AppAction>, Vec<AppEvent>, ()> {
    // Create the state struction
    let mut state = AppState::new();

    // Create the stream for updates coming from the UI side
    let ui_stream   = ui.get_updates();
    let cocoa_sink  = cocoa.get_input_sink();

    // Map the stream
    ui_stream
        .map(move |updates| {
            updates.into_iter()
                .flat_map(|update| state.map_update(update))
                .collect::<Vec<_>>()
        })
        .forward(cocoa_sink)
        .map(|_| ())
}
