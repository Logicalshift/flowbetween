use futures::prelude::*;
use futures::task::{waker, Waker, Context, ArcWake, Poll};

use std::pin::*;
use std::sync::*;

///
/// A priority future is a future that we can check if it has woken without actively polling it, and also update
/// the waker without polling it.
///
/// This allows the future to be used in a priority system: eg, polling it preferentially or deliberately not polling
/// it when higher priority tasks are available.
///
pub struct PriorityFuture<TFuture> {
    /// The future that is being prioritised
    future: TFuture,

    /// The waker for the 'outer' context (if it's not already woken)
    waker: Arc<Mutex<Option<Waker>>>,

    /// Boolean indicating whether or not the 'inner' future has woken us up
    is_ready: Arc<Mutex<bool>>
}

struct PriorityWaker {
    /// The waker for the 'outer' context (if it's not already woken)
    waker: Arc<Mutex<Option<Waker>>>,

    /// Boolean indicating whether or not the 'inner' future has woken us up
    is_ready: Arc<Mutex<bool>>
}

impl ArcWake for PriorityWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Mark as ready
        { (*arc_self.is_ready.lock().unwrap()) = true; }

        // Wake up the 'outer' future
        let waker = { (*arc_self.waker.lock().unwrap()).take() };
        waker.map(|waker| waker.wake());
    }
}

impl<TFuture: Future> From<TFuture> for PriorityFuture<TFuture> {
    ///
    /// Calling PriorityFuture::from(future) is the main way to create a new priority future
    ///
    fn from(future: TFuture) -> PriorityFuture<TFuture> {
        PriorityFuture {
            future:     future,
            waker:      Arc::new(Mutex::new(None)),
            is_ready:   Arc::new(Mutex::new(true))
        }
    }
}

impl<TFuture> PriorityFuture<TFuture> {
    ///
    /// Returns true if the inner future is ready to be polled
    ///
    pub fn is_ready(&self) -> bool {
        *self.is_ready.lock().unwrap()
    }

    ///
    /// Wakes the specified context when the future wakes up
    ///
    pub fn update_waker(&self, context: &Context) {
        (*self.waker.lock().unwrap()) = Some(context.waker().clone());
    }

    ///
    /// Retrieves the future as a pinned field (if we're pinned ourselves)
    ///
    fn pinned_future(self: Pin<&mut Self>) -> Pin<&mut TFuture> {
        // This is okay because `field` is pinned when `self` is.
        unsafe { self.map_unchecked_mut(|priority_future| &mut priority_future.future) }
    }
}

impl<TFuture: Future> Future for PriorityFuture<TFuture> {
    type Output = TFuture::Output;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        // No longer ready to poll
        (*self.is_ready.lock().unwrap()) = false;

        // Wake from this context
        self.update_waker(context);

        // Create a new context to poll from
        let inner_waker         = waker(Arc::new(PriorityWaker { waker: Arc::clone(&self.waker), is_ready: Arc::clone(&self.is_ready) }));
        let mut inner_context   = Context::from_waker(&inner_waker);

        // Poll the inner future and return its result
        let future = self.pinned_future();
        TFuture::poll(future, &mut inner_context)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::future;
    use futures::executor;
    use futures::channel::oneshot;

    #[test]
    fn is_initially_ready() {
        let future      = future::ready(0);
        let priority    = PriorityFuture::from(future);

        assert!(priority.is_ready());
    }

    #[test]
    fn not_ready_after_poll() {
        let future          = future::pending::<u32>();
        let mut priority    = PriorityFuture::from(future);

        assert!(priority.is_ready());
        executor::block_on(async { 
            future::poll_fn(|ctxt| {
                let _ = priority.poll_unpin(ctxt);
                Poll::Ready(0)
            }).await
        });
        assert!(!priority.is_ready());
    }

    #[test]
    fn ready_after_signal() {
        let (send, recv)    = oneshot::channel();
        let mut priority    = PriorityFuture::from(recv);

        assert!(priority.is_ready());
        executor::block_on(async { 
            future::poll_fn(|ctxt| {
                let _ = priority.poll_unpin(ctxt);
                Poll::Ready(0)
            }).await
        });
        assert!(!priority.is_ready());

        send.send(0).unwrap();
        assert!(priority.is_ready());
    }
}
