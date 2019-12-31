use super::core::*;
use super::event::*;
use super::super::controller::*;
use super::super::gather_stream::*;

use flo_stream::*;

use ::desync::*;
use futures::future;

use std::sync::*;

///
/// Returns a publisher for sending events to a UI session
///
pub fn ui_event_publisher<CoreController: 'static+Controller>(controller: Arc<CoreController>, core: Arc<Desync<UiSessionCore>>) -> Publisher<Vec<UiEvent>> {
    // Create the publisher
    let mut publisher = Publisher::new(100);

    // TODO: the pipe might read ahead of the events and queue several processing sessions which would be better handled as a single
    // session that evaluates when all of the events are available

    // Pipe events to the session core
    pipe_in(core, gather(publisher.subscribe()), move |core, mut events| {
        if events.len() > 0 {
            // Fetch the pending events (only hold the lock long enough to swap them out)
            // Suspend and resume updates before and after the pending list
            events.insert(0, UiEvent::SuspendUpdates);
            events.push(UiEvent::ResumeUpdates);

            // Dispatch the events
            let events = core.reduce_events(events);
            core.dispatch_event(events, &*controller);
        }

        Box::pin(future::ready(()))
    });

    publisher
}
