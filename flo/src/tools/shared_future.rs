use futures::task;
use futures::future;
use futures::prelude::*;
use futures::future::{BoxFuture};

use std::pin::*;
use std::sync::*;

///
/// The future that's being shared
///
struct SharedFutureCore<'a, TOutput: 'a+Send> {
    /// The future that is being shared
    future: Arc<Mutex<BoxFuture<'a, TOutput>>>,

    /// The waker for the future (as we use a custom context)
    waker: Option<task::Waker>
}

///
/// A waker that can be used to poll the future outside of the current context
///
struct SharedFutureWaker<'a, TOutput: 'a+Send> {
    /// A weak reference to the future that should be woken up
    core: Weak<Mutex<SharedFutureCore<'a, TOutput>>>
}

///
/// A future that can be polled as a normal future or checked from single-threaded contexts to move execution on
///
pub struct SharedFuture<TOutput: 'static+Send> {
    /// The core of this future
    core: Arc<Mutex<SharedFutureCore<'static, TOutput>>>
}

impl<TOutput: 'static+Send> Clone for SharedFuture<TOutput> {
    fn clone(&self) -> SharedFuture<TOutput> {
        SharedFuture {
            core: self.core.clone()
        }
    }
}

impl<TOutput: 'static+Send> SharedFuture<TOutput>
where TOutput: Clone {
    ///
    /// Turns a future into a SharedFuture
    ///
    pub fn new<TFuture>(future: TFuture) -> SharedFuture<TFuture::Output>
    where   TFuture:            'static+Send+Future<Output=TOutput> {
        // Pin the future
        let future = future.boxed();
        let future = Arc::new(Mutex::new(future));

        // Create the the core
        let core = SharedFutureCore {
            future: future,
            waker:  None
        };

        // Put the core into a new SharedFuture
        SharedFuture {
            core: Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Checks the future to see if the result is available yet, returning it if it is
    ///
    pub fn check(&self) -> task::Poll<TOutput> {
        // Retrieve the future to poll
        let future = {
            let core            = self.core.lock().unwrap();
            Arc::clone(&core.future)
        };

        // Create a context to poll the future in
        let shared_waker        = SharedFutureWaker { core: Arc::downgrade(&self.core) };
        let shared_waker        = task::waker(Arc::new(shared_waker));
        let mut shared_context  = task::Context::from_waker(&shared_waker);

        // Poll the future (locked separately to the core so we won't deadlock if the waker is triggered)
        let mut future          = future.lock().unwrap();
        let poll_result         = future.poll_unpin(&mut shared_context);

        // If the result was successful, replace the future in the core with one that's already ready
        if let task::Poll::Ready(result) = &poll_result {
            // Create a ready future
            let ready_future = future::ready(result.clone());

            // Substitute into the active future
            *future = ready_future.boxed();
        }

        poll_result
    }
}

impl<TOutput> Future for SharedFuture<TOutput>
where TOutput: 'static+Send {
    type Output = TOutput;

    fn poll(self: Pin<&mut Self>, context: &mut task::Context) -> task::Poll<Self::Output> {
        // Store the 'real' waker and retrieve the future to poll
        let future = {
            let mut core        = self.core.lock().unwrap();

            // Store the actual future waker
            core.waker          = Some(context.waker().clone());

            Arc::clone(&core.future)
        };

        // Create a context to poll the future in
        let shared_waker        = SharedFutureWaker { core: Arc::downgrade(&self.core) };
        let shared_waker        = task::waker(Arc::new(shared_waker));
        let mut shared_context  = task::Context::from_waker(&shared_waker);

        // Poll the future (locked separately to the core so we won't deadlock if the waker is triggered)
        let mut future          = future.lock().unwrap();
        let poll_result         = future.poll_unpin(&mut shared_context);
        poll_result
    }
}


impl<'a, TOutput> task::ArcWake for SharedFutureWaker<'a, TOutput>
where TOutput: Send {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Take the waker from the core
        let waker = {
            if let Some(core) = arc_self.core.upgrade() {
                let mut core = core.lock().unwrap();
                core.waker.take()
            } else {
                None
            }
        };

        // If there is a waker, trigger it
        waker.map(|waker| waker.wake());
    }
}
