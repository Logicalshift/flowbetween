use super::event::*;
use super::update::*;
use super::parked_future::*;
use super::http_user_interface::*;

use ui::*;
use ui::session::*;

use futures::*;
use futures::future;

use std::mem;
use std::sync::*;

///
/// Represents a session running on a HTTP connection 
/// 
pub struct HttpSession<CoreUi> {
    /// The core UI object
    http_ui: Arc<HttpUserInterface<CoreUi>>,

    /// The event sink for the UI
    input: Box<Future<Item=HttpEventSink, Error=()>>,

    /// The stream of events for the session (or None if it has been reset or not started yet)
    updates: Box<Future<Item=HttpUpdateStream, Error=()>>
}

impl<CoreUi: 'static+CoreUserInterface> HttpSession<CoreUi> {
    ///
    /// Creates a new session from a HTTP user interface
    /// 
    pub fn new(http_ui: Arc<HttpUserInterface<CoreUi>>) -> HttpSession<CoreUi> {
        let input   = Box::new(future::ok(http_ui.get_input_sink()));
        let updates = Box::new(future::ok(http_ui.get_updates()));

        HttpSession {
            http_ui:    http_ui,
            input:      input,
            updates:    updates
        }
    }

    ///
    /// Restarts the update stream (will regenerate the 'new UI' event, which is
    /// returned in the future return value).
    /// 
    pub fn restart_updates(&mut self) -> Box<Future<Item=Vec<Update>, Error=()>> {
        // Replace the update stream with a new one (the 'new session' even will start here)
        self.updates = Box::new(future::ok(self.http_ui.get_updates()));

        // Result is a future event from the updates
        let (updates, future_updates) = park_future();

        // We'll own the updates while we wait for this event
        let mut updates: Box<Future<Item=HttpUpdateStream, Error=()>> = Box::new(updates);
        mem::swap(&mut updates, &mut self.updates);

        let wait_for_update = updates.then(|updates| {
            // Poll for the next update
            let updates     = updates.unwrap();
            let mut updates = Some(updates);

            future::poll_fn(move || {
                let next_update = updates.as_mut().unwrap().poll();

                match next_update {
                    Ok(Async::Ready(result))    => Ok(Async::Ready((updates.take().unwrap(), result.unwrap_or(vec![])))),
                    Ok(Async::NotReady)         => Ok(Async::NotReady),
                    Err(derp)                   => Err(derp)
                }
            })
        });

        // Once the update is available, return ownership and supply the result to the caller
        let finish_update = wait_for_update.map(|(updates, result)| {
            future_updates.unpark(updates);
            result
        });

        Box::new(finish_update.fuse())
    }

    ///
    /// Sends some updates to this object and returns the resulting update
    /// 
    pub fn send_events(&mut self, events: Vec<Event>) -> Box<Future<Item=Vec<Update>, Error=()>> {
        // TODO: if the update stream is newly generated, we should wait for the initial 'new UI' event before polling for other events

        // We rely on the core UI only generating updates when we're polling
        // for them here.
        //
        // (If we get out of sync, we should be out of sync only for a single
        // event)
        let http_ui = Arc::clone(&self.http_ui);

        // Park our future input and updates
        let (input, future_input)       = park_future();
        let (updates, future_updates)   = park_future();

        // Take ownership of the future input and updates by replacing them with our parked values
        let mut input: Box<Future<Item=HttpEventSink, Error=()>>        = Box::new(input);
        let mut updates: Box<Future<Item=HttpUpdateStream, Error=()>>   = Box::new(updates);

        mem::swap(&mut input, &mut self.input);
        mem::swap(&mut updates, &mut self.updates);

        // Wait for the input and updates to be ready
        let input_and_updates = input.join(updates);

        // Once they are both ready, load the events into the input sink
        let load_events = input_and_updates.map(move |(mut input, mut updates)| {
            // Load the events
            for evt in events {
                match evt {
                    Event::UiRefresh => {
                        // Replace the updates with a new set if we get a refresh event
                        // TODO: defer this until after all the events are sent!
                        updates = http_ui.get_updates();
                    },

                    evt => {
                        // Send the other events to the input
                        input.start_send(evt).unwrap();
                    }
                };
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
                    Ok(Async::Ready(result))    => Ok(Async::Ready((updates.take().unwrap(), result.unwrap_or(vec![])))),
                    Ok(Async::NotReady)         => Ok(Async::NotReady),
                    Err(derp)                   => Err(derp)
                }
            }).map(move |(updates, result)| (input, updates, result))
        });
        
        // Once the update is ready, return the input and updates so we can send the next set of events and produce the result
        let finish_update = wait_for_update.map(move |(input, updates, result)| {
            // Return ownership of the input
            future_input.unpark(input);

            // Return ownership of the updates
            future_updates.unpark(updates);

            // Only return the result
            result
        });

        // finish_update is the result
        Box::new(finish_update.fuse())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::null_session::*;

    use futures::executor;

    #[test]
    fn will_return_update_for_event() {
        let null_session        = NullSession::new();
        let ui                  = UiSession::new(null_session);
        let http_ui             = HttpUserInterface::new(Arc::new(ui), "base/path".to_string());
        let mut http_session    = HttpSession::new(Arc::new(http_ui));

        let mut send_an_event   = executor::spawn(http_session.send_events(vec![Event::NewSession, Event::UiRefresh]));
        let updates             = send_an_event.wait_future();

        // Update should contain the new user interface message
        assert!(updates.unwrap().len() > 0);
    }

    #[test]
    fn will_return_update_for_next_event() {
        let null_session        = NullSession::new();
        let ui                  = UiSession::new(null_session);
        let http_ui             = HttpUserInterface::new(Arc::new(ui), "base/path".to_string());
        let mut http_session    = HttpSession::new(Arc::new(http_ui));

        let mut send_an_event   = executor::spawn(http_session.send_events(vec![Event::NewSession, Event::UiRefresh]));
        send_an_event.wait_future().unwrap();

        let mut send_another_event  = executor::spawn(http_session.send_events(vec![Event::Tick]));
        let updates                 = send_another_event.wait_future();

        // Second update will return but as it's a tick and nothing happens there will be no events
        assert!(updates.unwrap().len() == 0);
    }
}
