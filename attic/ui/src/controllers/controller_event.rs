use crate::control::*;

use futures::prelude::*;
use futures::task::{Waker, Poll, Context};

use std::mem;
use std::pin::*;
use std::sync::*;
use std::collections::{VecDeque};

///
/// Events that a controller can generate
///
#[derive(Clone, Debug, PartialEq)]
pub enum ControllerEvent {
    /// An action was generated from the UI
    Action(String, ActionParameter),
}

///
/// Stream to pass events from a controller to its runtime function
///
pub struct ControllerEventStream {
    core: Arc<Mutex<ControllerEventStreamCore>>
}

pub (crate) struct ControllerEventStreamCore {
    /// Events that have not been returned by the stream yet
    pending: VecDeque<ControllerEvent>,

    /// True if the stream has been closed and won't return any more items after the pending queue has been emptied
    closed: bool,

    /// Waker from the last time this stream returned that it was pending
    waker: Option<Waker>
}

impl ControllerEventStream {
    pub (crate) fn new() -> (Arc<Mutex<ControllerEventStreamCore>>, ControllerEventStream) {
        let core    = Arc::new(Mutex::new(ControllerEventStreamCore::new()));
        let stream  = ControllerEventStream { core: Arc::clone(&core) };

        (core, stream)
    }
}

impl Stream for ControllerEventStream {
    type Item = ControllerEvent;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<ControllerEvent>> {
        // Claim access to the core
        let mut core = self.core.lock().unwrap();

        // Any existing waker is invalidated
        core.waker = None;

        if let Some(next_item) = core.pending.pop_front() {
            // Return the next pending item if there is one
            Poll::Ready(Some(next_item))
        } else if core.closed {
            // Indicate that the stream is closed if anything 
            Poll::Ready(None)
        } else {
            // Awaken the stream when data becomes available (or it is closed)
            core.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

impl ControllerEventStreamCore {
    ///
    /// Creates a new core for the controller event stream
    ///
    fn new() -> ControllerEventStreamCore {
        ControllerEventStreamCore {
            pending:    VecDeque::new(),
            closed:     false,
            waker:      None
        }
    }
}

///
/// Functions that can be performed on an event stream core
///
pub (crate) trait ControllerEventStreamCoreActions {
    /// Sends one or more events to the core 
    fn post_events<ControllerEventIter: Iterator<Item=ControllerEvent>>(&self, events: ControllerEventIter);

    /// Takes the pending events from this stream and returns them (removing them from this stream)
    fn take_pending(&self) -> VecDeque<ControllerEvent>;

    /// Marks the core as closed
    fn close(&self);
}

impl ControllerEventStreamCoreActions for Arc<Mutex<ControllerEventStreamCore>> {
    /// Sends one or more events to the core 
    fn post_events<ControllerEventIter: Iterator<Item=ControllerEvent>>(&self, events: ControllerEventIter) {
        let wake = {
            let mut core = self.lock().unwrap();

            if !core.closed {
                core.pending.extend(events);
            }

            if core.pending.len() > 0 {
                // Take the core waker
                core.waker.take()
            } else {
                // If no events were pushed, we don't need to wake the core
                None
            }
        };

        // Wake once the lock has been released
        if let Some(wake) = wake {
            wake.wake()
        }
    }

    /// Takes the pending events from this stream and returns them (removing them from this stream)
    fn take_pending(&self) -> VecDeque<ControllerEvent> {
        let mut core = self.lock().unwrap();
        mem::take(&mut core.pending)
    }

    /// Marks the core as closed
    fn close(&self) {
        let wake = {
            let mut core = self.lock().unwrap();

            if !core.closed {
                // Close the core
                core.closed = true;

                // Take the core waker
                core.waker.take()
            } else {
                // If no events were pushed, we don't need to wake the core
                None
            }
        };

        // Wake once the lock has been released
        if let Some(wake) = wake {
            wake.wake()
        }
    }
}