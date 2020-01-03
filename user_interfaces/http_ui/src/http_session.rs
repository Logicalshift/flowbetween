use super::event::*;
use super::update::*;
use super::lazy_future::*;
use super::http_user_interface::*;

use ui::*;
use ui::session::*;
use flo_stream::*;
use flo_logging::*;

use futures::*;
use futures::future;
use futures::channel::oneshot;
use futures::task::{Poll};
use futures::future::{BoxFuture};

use std::mem;
use std::sync::*;

///
/// Represents a session running on a HTTP connection
///
pub struct HttpSession<CoreUi> {
    /// The publisher for this session
    log: LogPublisher,

    /// The core UI object
    http_ui: Arc<HttpUserInterface<CoreUi>>,

    /// The event sink for the UI
    input: BoxFuture<'static, WeakPublisher<Vec<Event>>>,

    /// The stream of events for the session (or None if it has been reset or not started yet)
    updates: BoxFuture<'static, HttpUpdateStream>
}

impl<CoreUi: 'static+CoreUserInterface+Send+Sync> HttpSession<CoreUi> {
    ///
    /// Creates a new session from a HTTP user interface
    ///
    pub fn new(http_ui: Arc<HttpUserInterface<CoreUi>>) -> HttpSession<CoreUi> {
        let input   = Box::pin(future::ready(http_ui.get_input_sink()));
        let updates = Box::pin(future::ready(http_ui.get_updates()));
        let log     = LogPublisher::new(module_path!());

        HttpSession {
            log:        log,
            http_ui:    http_ui,
            input:      input,
            updates:    updates
        }
    }

    ///
    /// Retrieves the log for this session
    ///
    pub fn log(&self) -> &LogPublisher {
        &self.log
    }

    ///
    /// Retrieves the HTTP user interface that this session is for
    ///
    pub fn http_ui(&self) -> Arc<HttpUserInterface<CoreUi>> {
        Arc::clone(&self.http_ui)
    }

    ///
    /// Retrieves the core UI that this session is for
    ///
    pub fn ui(&self) -> Arc<CoreUi> {
        self.http_ui.core()
    }

    ///
    /// Sleeps this session (stops any monitoring for events)
    ///
    pub fn fall_asleep(&mut self) {
        // Suspend the updates
        let http_ui = self.http_ui.clone();
        self.updates = Box::pin(LazyFuture::new(move || {
            future::ready(http_ui.get_updates())
        }));

        // Suspend the input
        let http_ui = self.http_ui.clone();
        self.input = Box::pin(LazyFuture::new(move || {
            future::ready(http_ui.get_input_sink())
        }));

        // TODO: way to signal to the caller that they should call 'restart_updates' here
        // (or make the first event from updates concatenate itself with the second event)
        //
        // At the moment, this should only happen if the websocket connection establishes
        // but fails later on for some reason without restarting the session.
        //
        // This is needed because the update stream always generates an update for
        // every event, but also has an initial update with the 'new HTML' message
        // in it. Something sending an event while the 'new HTML' message is
        // waiting will get slightly out of sync (though the 'on demand' nature
        // of the update stream means it'll only be out of sync for one event)
    }

    ///
    /// Restarts the update stream (will regenerate the 'new UI' event, which is
    /// returned in the future return value).
    ///
    pub fn restart_updates(&mut self) -> BoxFuture<'static, Vec<Update>> {
        // Replace the update stream with a new one (the 'new session' even will start here)
        self.updates = Box::pin(future::ready(self.http_ui.get_updates()));

        // Result is a future event from the updates
        let (future_updates, updates) = oneshot::channel();

        // We'll own the updates while we wait for this event
        let mut updates: BoxFuture<'static, HttpUpdateStream> = Box::pin(updates.map(|res| res.unwrap()));
        mem::swap(&mut updates, &mut self.updates);

        let wait_for_update = updates.then(|updates| {
            // Poll for the next update
            let updates     = updates;
            let mut updates = Some(updates);

            future::poll_fn(move |context| {
                let next_update = updates.as_mut().unwrap().poll_next_unpin(context);

                match next_update {
                    Poll::Ready(Some(Ok(result)))   => Poll::Ready((updates.take().unwrap(), result)),
                    Poll::Ready(Some(Err(_)))       => Poll::Ready((updates.take().unwrap(), vec![])),
                    Poll::Ready(None)               => Poll::Ready((updates.take().unwrap(), vec![])),
                    Poll::Pending                   => Poll::Pending,
                }
            })
        });

        // Once the update is available, return ownership and supply the result to the caller
        let finish_update = wait_for_update.map(|(updates, result)| {
            future_updates.send(updates).ok();
            result
        });

        Box::pin(finish_update.fuse())
    }

    ///
    /// Sends some updates to this object and returns the resulting update
    ///
    pub fn send_events(&mut self, events: Vec<Event>) -> BoxFuture<'static, Vec<Update>> {
        // TODO: if the update stream is newly generated, we should wait for the initial 'new UI' event before polling for other events

        // We rely on the core UI only generating updates when we're polling
        // for them here.
        //
        // (If we get out of sync, we should be out of sync only for a single
        // event)
        let http_ui = Arc::clone(&self.http_ui);

        // Park our future input and updates
        let (future_input, input)       = oneshot::channel();
        let (future_updates, updates)   = oneshot::channel();

        // Take ownership of the future input and updates by replacing them with our parked values
        let mut input: BoxFuture<'static, WeakPublisher<Vec<Event>>>    = Box::pin(input.map(|input| input.unwrap()));
        let mut updates: BoxFuture<'static, HttpUpdateStream>           = Box::pin(updates.map(|updates| updates.unwrap()));

        mem::swap(&mut input, &mut self.input);
        mem::swap(&mut updates, &mut self.updates);

        // Wait for the input and updates to be ready
        let input_and_updates = future::join(input, updates);

        // Once they are both ready, load the events into the input sink
        let load_events = input_and_updates.map(move |(mut input, mut updates)| {
            let mut refresh = false;

            // Load the events
            let mut to_send = vec![];
            for evt in events {
                match evt {
                    Event::UiRefresh | Event::NewSession => {
                        // Replace the updates with a new set if we get a refresh event
                        refresh = true;
                    },

                    evt => {
                        // Send the other events to the input
                        to_send.push(evt);
                    }
                };
            }

            let wait_for_publish = input.publish(to_send);

            // Restart the update queue if there's a refresh event
            if refresh {
                // Forces the 'new session' update to get regenerated
                updates = http_ui.get_updates();
            }

            // Pass the input and updates forwards
            (input, wait_for_publish, updates)
        });

        // Wait for the events to complete once this is done
        let wait_for_events = load_events.then(|(input, wait_for_publish, updates): (_, _, HttpUpdateStream)| {
            wait_for_publish.then(|_| future::ready((input, updates)))
        });

        // Once the events are flushed, we need to wait for the stream to send us an update
        let wait_for_update = wait_for_events.then(|events| {
            let (input, updates) = events;

            let mut updates = updates;
            async move {
                let result = updates.next().await
                    .unwrap_or_else(|| Ok(vec![]))
                    .unwrap();
                (input, updates, result)
            }
        });

        // Once the update is ready, return the input and updates so we can send the next set of events and produce the result
        let finish_update = wait_for_update.map(move |(input, updates, result)| {
            // Return ownership of the input
            future_input.send(input).ok();

            // Return ownership of the updates
            future_updates.send(updates).ok();

            // Only return the result
            result
        });

        // finish_update is the result
        Box::pin(finish_update.fuse())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::null_session::*;

    use futures::executor;

    #[test]
    fn will_return_update_for_event() {
        let thread_pool                 = executor::ThreadPool::new().unwrap();
        let null_session                = NullSession::new();
        let (ui, ui_run_loop)           = UiSession::new(null_session);
        let (http_ui, http_run_loop)    = HttpUserInterface::new(Arc::new(ui), "base/path".to_string());
        let mut http_session            = HttpSession::new(Arc::new(http_ui));

        thread_pool.spawn_ok(ui_run_loop);
        thread_pool.spawn_ok(http_run_loop);

        let send_an_event               = http_session.send_events(vec![Event::NewSession, Event::UiRefresh]);
        let updates                     = executor::block_on(send_an_event);

        // Update should contain the new user interface message
        assert!(updates.len() > 0);
    }

    #[test]
    fn will_return_update_for_next_event() {
        let thread_pool                 = executor::ThreadPool::new().unwrap();
        let null_session                = NullSession::new();
        let (ui, ui_run_loop)           = UiSession::new(null_session);
        let (http_ui, http_run_loop)    = HttpUserInterface::new(Arc::new(ui), "base/path".to_string());
        let mut http_session            = HttpSession::new(Arc::new(http_ui));

        thread_pool.spawn_ok(ui_run_loop);
        thread_pool.spawn_ok(http_run_loop);

        let send_an_event               = http_session.send_events(vec![Event::NewSession, Event::UiRefresh]);
        executor::block_on(send_an_event);

        let send_another_event          = http_session.send_events(vec![Event::Tick]);
        let updates                     = executor::block_on(send_another_event);

        // Second update will return but as it's a tick and nothing happens there will be no events
        assert!(updates.len() == 0);
    }
}
