use super::core::*;
use super::event::*;
use super::super::controller::*;

use desync::*;
use futures::*;
use futures::task;

use std::mem;
use std::sync::*;

///
/// The event sink works with a UI session. When events arrive, they can be sent
/// to one of these
/// 
pub struct UiEventSink {
    /// The core controller that will be the target for these events
    controller: Arc<dyn Controller>,

    /// The core that is affected by these events
    core: Arc<Desync<UiSessionCore>>,

    /// The events that are waiting to be processed (empty if the sink is idle)
    pending_events: Arc<Mutex<Vec<UiEvent>>>,

    /// True while we've got an async request to process events queued
    waiting_for_events: Arc<Mutex<bool>>
}

impl UiEventSink {
    ///
    /// Creates a new event sink
    /// 
    pub fn new<CoreController: 'static+Controller>(controller: Arc<CoreController>, core: Arc<Desync<UiSessionCore>>) -> UiEventSink {
        UiEventSink {
            controller:             controller,
            core:                   core,
            pending_events:         Arc::new(Mutex::new(vec![])),
            waiting_for_events:     Arc::new(Mutex::new(false))
        }
    }
}

impl Sink for UiEventSink {
    type SinkItem   = Vec<UiEvent>;
    type SinkError  = ();

    fn start_send(&mut self, item: Vec<UiEvent>) -> StartSend<Vec<UiEvent>, ()> {
        if item.len() == 0 {
            // Edge case: no events are able to be sent
            Ok(AsyncSink::Ready)
        } else {
            // Get the events that are pending
            let pending_events  = self.pending_events.clone();
            let mut pending     = pending_events.lock().unwrap();

            if pending.len() == 0 {
                // No events are pending, so we need to wake the core
                *pending = item;

                // Need to send some stuff to the core to finish processing the event
                let controller          = Arc::clone(&self.controller);
                let pending_events      = self.pending_events.clone();
                let waiting_for_events  = Arc::clone(&self.waiting_for_events);

                // Setting the 'waiting for events' causes poll_complete to indicate that we're still busy
                *waiting_for_events.lock().unwrap() = true;

                self.core.desync(move |core| {
                    // Fetch the pending events (only hold the lock long enough to swap them out)
                    let mut events      = vec![];
                    {
                        let mut pending = pending_events.lock().unwrap();

                        mem::swap(&mut *pending, &mut events);
                    }

                    // Suspend and resume updates before and after the pending list
                    events.insert(0, UiEvent::SuspendUpdates);
                    events.push(UiEvent::ResumeUpdates);

                    // Dispatch the events
                    let events = core.reduce_events(events);
                    core.dispatch_event(events, &*controller);

                    // No longer waiting for events
                    *waiting_for_events.lock().unwrap() = false;
                });

                // Item went to the sink
                Ok(AsyncSink::Ready)
            } else {
                // Events are already pending, so we just add this new set onto the old set
                pending.extend(item);

                // Item went to the sink
                Ok(AsyncSink::Ready)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        if *self.waiting_for_events.lock().unwrap() == false {
            // All events have been pushed to the core (though they may still be running)
            Ok(Async::Ready(()))
        } else {
            // Generate a task and defer until the core is available again
            let task = task::current();
            self.core.desync(move |_| {
                // The event we were expecting will be retired at this point, so signal the task
                // New events might be present so the next poll might also be not ready
                task.notify();
            });

            // Events are still waiting to be dispatched/being dispatched
            Ok(Async::NotReady)
        }
    }
}
