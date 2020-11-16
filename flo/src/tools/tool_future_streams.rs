use super::tool_input::*;
use super::tool_action::*;
use crate::model::*;

use flo_animation::*;

use futures::prelude::*;
use futures::task;
use futures::task::{Poll, Waker};
use futures::stream;
use futures::stream::{BoxStream};

use std::iter;
use std::pin::*;
use std::sync::*;
use std::collections::{VecDeque};

///
/// Shared core for the tool streams
///
pub (super) struct ToolStreamCore<ToolData> {
    /// Any input waiting to be consumed
    pending: VecDeque<ToolData>,

    /// Set to true if the stream is closed
    closed: bool,

    // If the stream was not ready last time it was polled, the waker to start it up again 
    waker: Option<Waker>
}

///
/// Sends tool actions from a ToolFuture to the rest of the FlowBetween runtime
///
pub struct ToolActionPublisher<ToolData> {
    core: Arc<Mutex<ToolStreamCore<ToolAction<ToolData>>>>
}

///
/// Stream of actions published by the action publisher
///
pub (super) struct ToolActionStream<ToolData, ToolFuture> {
    action_core:    Arc<Mutex<ToolStreamCore<ToolAction<ToolData>>>>,
    input_core:     Arc<Mutex<ToolStreamCore<ToolInput<ToolData>>>>,

    future:         Option<ToolFuture>
}

///
/// Buffering stream that sends tool input to a tool future
///
pub struct ToolInputStream<ToolData> {
    core: Arc<Mutex<ToolStreamCore<ToolInput<ToolData>>>>
}

impl<ToolData> Stream for ToolInputStream<ToolData> {
    type Item = ToolInput<ToolData>;

    fn poll_next(self: Pin<&mut Self>, context: &mut task::Context) -> Poll<Option<ToolInput<ToolData>>> {
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

impl<ToolData, ToolFuture> Stream for ToolActionStream<ToolData, ToolFuture> 
where ToolFuture: Unpin+Future<Output=()> {
    type Item = ToolAction<ToolData>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut task::Context) -> Poll<Option<ToolAction<ToolData>>> {
        // Poll the tool future
        if let Some(Poll::Ready(())) = self.future.as_mut().map(|future| future.poll_unpin(context)) {
            self.future = None;
        }

        // Claim access to the core
        let mut action_core = self.action_core.lock().unwrap();

        // Any existing waker is invalidated
        action_core.waker = None;

        if let Some(next_item) = action_core.pending.pop_front() {
            // Return the next pending item if there is one
            Poll::Ready(Some(next_item))
        } else if action_core.closed {
            // Indicate that the stream is closed if anything 
            Poll::Ready(None)
        } else {
            // Awaken the stream when data becomes available (or it is closed)
            action_core.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

impl<ToolData, ToolFuture> ToolActionStream<ToolData, ToolFuture> 
where ToolFuture: Unpin+Future<Output=()> {
    ///
    /// Sets the future that generates results for this stream
    ///
    pub (super) fn set_future(&mut self, future: ToolFuture)  {
        self.future = Some(future);
    }
}

///
/// Creates a new tool action stream
///
pub (super) fn create_tool_action_stream<ToolData, ToolFuture>(input_core: &Arc<Mutex<ToolStreamCore<ToolInput<ToolData>>>>) -> (ToolActionStream<ToolData, ToolFuture>, Arc<Mutex<ToolStreamCore<ToolAction<ToolData>>>>) {
    // Create the core and wrap it in a mutex
    let core = ToolStreamCore {
        pending:    VecDeque::new(),
        closed:     false,
        waker:      None
    };
    let core = Arc::new(Mutex::new(core));

    // Create the stream
    let stream = ToolActionStream {
        action_core:    core.clone(),
        input_core:     input_core.clone(),
        future:         None
    };

    (stream, core)
}


///
/// Creates a tool action publisher to pass into the future
///
pub (super) fn create_tool_action_publisher<ToolData>(core: &Arc<Mutex<ToolStreamCore<ToolAction<ToolData>>>>) -> ToolActionPublisher<ToolData> {
    ToolActionPublisher {
        core: Arc::clone(core)
    }
}

///
/// Creates a new tool input stream and its core
///
pub (super) fn create_tool_input_stream<ToolData>() -> (ToolInputStream<ToolData>, Arc<Mutex<ToolStreamCore<ToolInput<ToolData>>>>) {
    // Create the core and wrap it in a mutex
    let core = ToolStreamCore {
        pending:    VecDeque::new(),
        closed:     false,
        waker:      None
    };
    let core = Arc::new(Mutex::new(core));

    // Create the stream
    let stream = ToolInputStream {
        core: core.clone()
    };

    (stream, core)
}

///
/// Sends some tool input to the input stream for processing
///
pub (super) fn send_tool_stream<ToolData, InputIterator>(input_stream: &Arc<Mutex<ToolStreamCore<ToolData>>>, input: InputIterator)
where InputIterator: IntoIterator<Item=ToolData> {
    // Send the input and retrieve the waker if there is one
    let waker = {
        // Fetch the core
        let mut core = input_stream.lock().unwrap();

        // Send the input
        core.pending.extend(input);

        // Take any waker that's pending
        core.waker.take()
    };

    // Wake anything that's waiting for input from the stream
    waker.map(|waker| waker.wake());
}

///
/// Marks a tool input stream as closed
///
pub (super) fn close_tool_stream<ToolData>(input_stream: &Arc<Mutex<ToolStreamCore<ToolData>>>) {
    // Send the input and retrieve the waker if there is one
    let waker = {
        // Fetch the core
        let mut core = input_stream.lock().unwrap();

        // Mark as closed
        core.closed = true;

        // Take any waker that's pending
        core.waker.take()
    };

    // Wake anything that's waiting for input from the stream
    waker.map(|waker| waker.wake());
}
