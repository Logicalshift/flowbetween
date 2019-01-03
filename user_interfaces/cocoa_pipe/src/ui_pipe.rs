use super::action::*;
use super::app_state::*;

use flo_ui::session::*;

use futures::*;

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<UiStream, CocoaSink>(ui_stream: UiStream, cocoa_sink: CocoaSink) -> impl Future<Item=()>
where   UiStream:   Stream<Item=Vec<UiUpdate>, Error=()>,
        CocoaSink:  Sink<SinkItem=Vec<AppAction>, SinkError=()> {
    // Create the state struction
    let mut state = AppState::new();

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
