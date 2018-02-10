use super::core::*;
use super::event::*;
use super::super::controller::*;

use desync::*;
use futures::*;

use std::sync::*;

///
/// The event sink works with a UI session. When events arrive, they can be sent
/// to one of these
/// 
pub struct EventSink<CoreController: Controller> {
    /// The core controller that will be the target for these events
    controller: Arc<CoreController>,

    /// The core that is affected by these events
    core: Arc<Desync<UiSessionCore>>,

    /// ID assigned to the next event
    next_event: Mutex<usize>,

    /// The event that was most recently retired for this sink
    last_finished_event: Arc<Mutex<usize>>
}

impl<CoreController: Controller> EventSink<CoreController> {
    ///
    /// Creates a new event sink
    /// 
    pub fn new(controller: Arc<CoreController>, core: Arc<Desync<UiSessionCore>>) -> EventSink<CoreController> {
        EventSink {
            controller:             controller,
            core:                   core,
            next_event:             Mutex::new(0),
            last_finished_event:    Arc::new(Mutex::new(0))
        }
    }
}

impl<CoreController: Controller+'static> Sink for EventSink<CoreController> {
    type SinkItem   = UiEvent;
    type SinkError  = ();

    fn start_send(&mut self, item: UiEvent) -> StartSend<UiEvent, ()> {
        // Assign an ID to this event
        let event_id: usize = {
            let mut next_event  = self.next_event.lock().unwrap();
            let event_id        = *next_event;
            (*next_event)       += 1;

            event_id
        };

        // Clone the controller
        let controller          = Arc::clone(&self.controller);
        let last_finished_event = Arc::clone(&self.last_finished_event);

        // Send to the core (which acts as our sink)
        self.core.async(move |core| {
            // Dispatch the event
            core.dispatch_event(item, &*controller);

            // Retire the event
            let mut last_finished_event = last_finished_event.lock().unwrap();
            if *last_finished_event < event_id {
                *last_finished_event = event_id;
            }
        });

        // Item went to the sink
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        // We're ready
        Ok(Async::Ready(()))
    }
}
