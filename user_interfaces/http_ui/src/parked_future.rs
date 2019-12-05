use futures::*;
use futures::task;
use futures::task::Task;

use std::sync::*;

///
/// A future that will eventually have a value (this seems to be a feature missing from
/// the futures library)
///
pub struct ParkedFuture<TResult> {
    /// Core is shared between our parked future and its setter
    core: Arc<Mutex<ParkedFutureCore<TResult>>>
}

///
/// Unparks a parked future with a value
///
pub struct FutureUnparker<TResult> {
    /// Core is shared between our parked future and its setter
    core: Arc<Mutex<ParkedFutureCore<TResult>>>
}

///
/// Represents a parked future
///
struct ParkedFutureCore<TResult> {
    wait:       Option<Task>,
    result:     Option<TResult>,
    unparked:   bool
}

impl<TResult> Future for ParkedFuture<TResult> {
    type Item   = TResult;
    type Error  = ();

    fn poll(&mut self) -> Poll<TResult, ()> {
        let mut core = self.core.lock().unwrap();

        match core.result.take() {
            Some(val)   => Ok(Async::Ready(val)),
            None        => {
                core.wait = Some(task::current());
                Ok(Async::NotReady)
            }
        }
    }
}

impl<TResult> FutureUnparker<TResult> {
    ///
    /// Unparks this future and sets it to the specified result
    ///
    pub fn unpark(self, result: TResult) {
        let mut core = self.core.lock().unwrap();

        core.result     = Some(result);
        core.unparked   = true;
        core.wait.as_mut().map(|wait| wait.notify());
    }
}

impl<TResult> Drop for FutureUnparker<TResult> {
    fn drop(&mut self) {
        let core = self.core.lock().unwrap();

        if !core.unparked {
            // The future will never complete if the unparker is dropped without being activated
            panic!("Futures must be unparked before the unparker can be dropped");
        }
    }
}

///
/// Creates a parked future and an unparker that can be used to set its value
///
pub fn park_future<TResult>() -> (ParkedFuture<TResult>, FutureUnparker<TResult>) {
    // Create the core that we'll share
    let core = Arc::new(Mutex::new(ParkedFutureCore { wait: None, result: None, unparked: false }));

    // Turn into a future and an unparker
    let future      = ParkedFuture      { core: Arc::clone(&core) };
    let unparker    = FutureUnparker    { core: core };

    (future, unparker)
}
