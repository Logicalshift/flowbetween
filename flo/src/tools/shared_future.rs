use futures::task;
use futures::prelude::*;

use std::pin::*;
use std::sync::*;

///
/// The future that's being shared
///
struct SharedFutureCore<TFuture: Send+Future> {
    /// The future that is being shared
    future: Arc<Mutex<Pin<Box<TFuture>>>>,

    /// The waker for the future (as we use a custom context)
    waker: Option<task::Waker>
}

///
/// A waker that will
///
struct SharedFutureWaker<TFuture: Send+Future> {
    /// A weak reference to the future that should be woken up
    core: Weak<Mutex<SharedFutureCore<TFuture>>>
}

///
/// A future that can be polled as a normal future or checked from single-threaded contexts to move execution on
///
pub struct SharedFuture<TFuture: 'static+Send+Future> {
    /// The core of this future
    core: Arc<Mutex<SharedFutureCore<TFuture>>>
}

impl<TFuture: 'static+Send+Future> Clone for SharedFuture<TFuture> {
    fn clone(&self) -> SharedFuture<TFuture> {
        SharedFuture {
            core: self.core.clone()
        }
    }
}

impl<TFuture: 'static+Send+Future> SharedFuture<TFuture> {
    ///
    /// Turns a future into a SharedFuture
    ///
    pub fn new(future: TFuture) -> SharedFuture<TFuture> {
        // Pin the future
        let future = Box::pin(future);
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
    pub fn check(&self) -> task::Poll<TFuture::Output> {
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
        let poll_result         = TFuture::poll(future.as_mut(), &mut shared_context);
        poll_result
    }
}

impl<TFuture> Future for SharedFuture<TFuture>
where TFuture: 'static+Send+Future {
    type Output = TFuture::Output;

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
        let poll_result         = TFuture::poll(future.as_mut(), &mut shared_context);
        poll_result
    }
}


impl<TFuture> task::ArcWake for SharedFutureWaker<TFuture>
where TFuture: Send+Future {
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
