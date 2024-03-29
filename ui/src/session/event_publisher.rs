use super::core::*;
use super::event::*;
use super::priority_future::*;
use crate::controller::*;

use flo_stream::*;

use ::desync::*;
use futures::future;
use futures::task;
use futures::task::{Poll};
use futures::prelude::*;

use std::sync::*;

///
/// Polls the runtime if needed, returning true if the runtime was woken up
///
fn poll_runtime(maybe_runtime: &mut Option<PriorityFuture<impl Unpin+Future<Output=()>>>, context: &mut task::Context) -> bool {
    let mut woke_up = false;

    if let Some(runtime) = maybe_runtime {
        // Update the context in the runtime (in case it's not ready)
        runtime.update_waker(context);

        // Poll until it's no longer ready
        while runtime.is_ready() {
            woke_up = true;

            match runtime.poll_unpin(context) {
                Poll::Ready(_) => {
                    // Unset the runtime and stop if it completes
                    *maybe_runtime = None;
                    return woke_up;
                }

                Poll::Pending => { }
            }
        }
    }

    woke_up
}

///
/// The main UI event loop
///
pub fn ui_event_loop<CoreController: 'static+Controller>(controller: Weak<CoreController>, mut ui_events: WeakPublisher<Vec<UiEvent>>, runtime: impl 'static+Send+Unpin+Future<Output=()>, core: Weak<Desync<UiSessionCore>>) -> impl Unpin+Future<Output=()> {
    // Subscribe to the UI events
    let mut ui_event_subscriber = ui_events.subscribe();
    let mut runtime             = Some(PriorityFuture::from(runtime));

    async move {
        // Initial runtime poll before the main loop starts
        let runtime_poller  = &mut runtime;
        future::poll_fn(move |context| {
            poll_runtime(runtime_poller, context);
            Poll::Ready(())
        }).await;

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
            let runtime_poller  = &mut runtime;

            let mut next_events = ui_event_subscriber.next();
            let next_events     = future::poll_fn(move |context| {
                if poll_runtime(runtime_poller, context) {
                    // If the runtime woke up, generate a tick event
                    Poll::Ready(Some(vec![UiEvent::Tick]))
                } else {
                    next_events.poll_unpin(context)
                }
            });
            let next_events     = next_events.await;
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
                let runtime_poller              = &mut runtime;

                let (subscriber, more_events)   = future::poll_fn(move |context| {
                    // Give the runtime a chance to run first
                    poll_runtime(runtime_poller, context);

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

                    // Create the dispatcher future
                    let mut dispatcher  = core.future_sync(move |core| {
                        let core_events = core.reduce_events(core_events);
                        async move { core.dispatch_event(core_events, &*core_controller).await }.boxed()
                    });

                    // While it's running, also poll the runtime
                    let runtime_poller  = &mut runtime;
                    let dispatcher      = future::poll_fn(move |context| {
                        poll_runtime(runtime_poller, context);
                        dispatcher.poll_unpin(context)
                    });

                    dispatcher.await.ok();
                } else {
                    // Ran out of events to process
                    break;
                }
            }

            // === State: returning to idle
            
            // Resume updates after the events we just received
            let core_controller = Arc::clone(&controller);
            core.future_sync(move |core| {
                async move { core.dispatch_event(vec![UiEvent::ResumeUpdates], &*core_controller).await }.boxed()
            }).await.ok();
        }
    }.boxed()
}
