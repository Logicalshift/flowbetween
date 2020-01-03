use super::super::gtk_event::*;

use flo_stream::*;

use ::desync::*;
use futures::prelude::*;

use std::sync::*;

pub type GtkEventSink = Arc<Desync<WeakPublisher<GtkEvent>>>;

///
/// Sends an event immediately to an event sink
///
pub fn publish_event(sink: &GtkEventSink, event: GtkEvent) {
    let _ = sink.future(move |publisher| async move {
        publisher.publish(event).await
    }.boxed());
}
