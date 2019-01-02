use super::action::*;

use flo_ui::session::*;

use futures::*;

///
/// Pipes UI updates to a Cocoa UI action sink
///
pub fn pipe_ui_updates<UiStream, CocoaSink>(ui_stream: UiStream, cocoa_sink: CocoaSink) -> impl Future<Item=()>
where   UiStream:   Stream<Item=Vec<UiUpdate>, Error=()>,
        CocoaSink:  Sink<SinkItem=Vec<AppAction>, SinkError=()> {
    ui_stream
        .map(|updates| vec![])
        .forward(cocoa_sink)
        .map(|_| ())
}