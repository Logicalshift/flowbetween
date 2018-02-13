use super::event::*;
use super::update::*;
use super::http_user_interface::*;

use ui::*;
use ui::session::*;

use futures::*;
use futures::future;
use futures::stream;
use futures::executor;
use futures::executor::Spawn;

use std::mem;

///
/// Represents a session running on a HTTP connection 
/// 
pub struct HttpSession<CoreUi> {
    /// The core UI object
    http_ui: HttpUserInterface<CoreUi>,

    /// The event sink for the UI
    input: Box<Future<Item=HttpEventSink, Error=()>>,

    /// The stream of events for the session (or None if it has been reset or not started yet)
    updates: Box<Future<Item=HttpUpdateStream, Error=()>>
}

impl<CoreUi: 'static+CoreUserInterface> HttpSession<CoreUi> {
    ///
    /// Creates a new session from a HTTP user interface
    /// 
    pub fn new(http_ui: HttpUserInterface<CoreUi>) -> HttpSession<CoreUi> {
        let input   = Box::new(future::ok(http_ui.get_input_sink()));
        let updates = Box::new(future::ok(http_ui.get_updates()));

        HttpSession {
            http_ui:    http_ui,
            input:      input,
            updates:    updates
        }
    }

    ///
    /// Restarts the update stream (will regenerate the 'new UI' event)
    /// 
    pub fn restart_updates(&mut self) {
        self.updates = Box::new(future::ok(self.http_ui.get_updates()));
    }

    ///
    /// Sends some updates to this object and returns the resulting update
    /// 
    pub fn send_events(&mut self, events: Vec<Event>) -> Box<Future<Item=Vec<Update>, Error=()>> {
        // We rely on the core UI only generating updates when we're polling
        // for them here.
        //
        // (If we get out of sync, we should be out of sync only for a single
        // event)

        // Take ownership of the future input and upates. These errors will
        // replace them while this function executes but will be gone by the
        // time we finish
        let mut input: Box<Future<Item=HttpEventSink, Error=()>>        = Box::new(future::err(()));
        let mut updates: Box<Future<Item=HttpUpdateStream, Error=()>>   = Box::new(future::err(()));

        mem::swap(&mut input, &mut self.input);
        mem::swap(&mut updates, &mut self.updates);

        // Wait for the input and updates to be ready
        let input_and_updates = input.join(updates);

        // Once they are both ready, load the events into the input sink
        let load_events = input_and_updates.map(move |(mut input, updates)| {
            // Load the events
            for evt in events {
                input.start_send(evt).unwrap();
            }

            // Pass the input and updates forwards
            (input, updates)
        });

        // Wait for the events to complete once this is done
        let wait_for_events = load_events.then(|events| {
            // Going to assume no errors here
            let (input, updates) = events.unwrap();

            input.flush().map(move |input| (input, updates))
        });

        // Once the events are flushed, we need to wait for the stream to send us an update
        let wait_for_update = wait_for_events.then(|events| {
            // Still assuming we get no errors
            let (input, updates) = events.unwrap();

            let mut updates = Some(updates);
            future::poll_fn(move || {
                // Fetch the next update
                // We rely on the fact the update stream is lazy: there's no update waiting until we start polling, so this is the update for the events we just sent
                let next_update = updates.as_mut().unwrap().poll();

                match next_update {
                    Ok(Async::Ready(result))    => Ok(Async::Ready((updates.take(), result.unwrap_or(vec![])))),
                    Ok(Async::NotReady)         => Ok(Async::NotReady),
                    Err(derp)                   => Err(derp)
                }
            }).map(move |(updates, result)| (input, updates, result))
        });
        
        // Once the update is ready, return the input and updates so we can send the next set of events and produce the result
        let finish_update = wait_for_update.map(|(input, updates, result)| {
            // TODO: Store/notify the input
            // (Really want a way to park/unpark a future but the futures library doesn't have a thing like that that I can see)

            // TODO: Store/notify the updates

            // Only return the result
            result
        });

        // finish_update is the result
        Box::new(finish_update)
    }
}
