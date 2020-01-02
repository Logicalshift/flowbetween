use super::core::*;
use super::event::*;
use super::super::controller::*;
use super::super::gather_stream::*;

use flo_stream::*;

use ::desync::*;
use futures::future;
use futures::task::{Poll};
use futures::prelude::*;

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

///
/// The main UI event loop
///
pub async fn ui_event_loop<CoreController: 'static+Controller>(controller: &Arc<CoreController>, mut ui_events: WeakPublisher<Vec<UiEvent>>, core: &Arc<Desync<UiSessionCore>>) {
    // Subscribe to the UI events
    let core                    = Arc::downgrade(core);
    let controller              = Arc::downgrade(controller);
    let mut ui_event_subscriber = ui_events.subscribe();

    // Start the main UI loop
    loop {
        // === State: idle (wait for an event to arrive)
        
        let core        = match core.upgrade() {
            None                => { break; }
            Some(core)          => core
        };
        let controller  = match controller.upgrade() {
            None                => { break; }
            Some(controller)    => controller
        };

        // Wait for the next event to arrive
        let next_events     = ui_event_subscriber.next().await;
        let mut next_events = match next_events {
            None                => { break; }
            Some(next_events)   => next_events
        };

        // === State: processing events
        next_events.insert(0, UiEvent::SuspendUpdates);
        let mut next_events = Some(next_events);

        loop {
            // Read as many events as possible from the queue
            let mut poll_events             = next_events;
            let mut poll_subscriber         = Some(ui_event_subscriber);

            let (subscriber, more_events)   = future::poll_fn(move |context| {
                // Add to the list of events as long as the subscriber is ready (we want to process as many as possible before resuming the UI)
                while let Poll::Ready(Some(more_events)) = poll_subscriber.as_mut().unwrap().poll_next_unpin(context) {
                    poll_events = match poll_events.take() {
                        Some(mut events) => {
                            events.extend(more_events);
                            Some(events)
                        },
                        None => { 
                            Some(more_events)
                        }
                    }
                }

                // Return ownership of the subscriber and the events to the main event loop
                Poll::Ready((poll_subscriber.take().unwrap(), poll_events.take()))
            }).await;

            // UI event subscriber no longer owned by the poll function
            ui_event_subscriber = subscriber;
            next_events         = more_events;

            // Dispatch the events to the core
            let core_events     = next_events.take();

            if let Some(core_events) = core_events {
                let core_controller = Arc::clone(&controller);

                core.future(move |core| {
                    let core_events = core.reduce_events(core_events);
                    core.dispatch_event(core_events, &*core_controller);

                    future::ready(()).boxed()
                }).await.ok();
            } else {
                // Ran out of events to process
                break;
            }
        }

        // === State: returning to idle
        
        // Resume updates after the events we just received
        let core_controller = Arc::clone(&controller);
        core.future(move |core| {
            core.dispatch_event(vec![UiEvent::ResumeUpdates], &*core_controller);
            future::ready(()).boxed()
        }).await.ok();
    }
}
