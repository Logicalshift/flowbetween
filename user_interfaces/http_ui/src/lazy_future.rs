use futures::*;
use futures::task::{Poll, Context};

use std::pin::*;
use std::sync::*;

struct LazyFutureCore<Item, F: Future<Output=Item>, MakeFuture: FnOnce() -> F> {
    /// Function to create a future
    make_future: Option<MakeFuture>,

    /// The future if it is real
    the_future: Option<F>
}

///
/// A lazy future creates an 'actual' future when polled but otherwise creates nothing
///
pub struct LazyFuture<Item, F: Future<Output=Item>, MakeFuture: FnOnce() -> F> {
    core: Mutex<LazyFutureCore<Item, F, MakeFuture>>
}

impl<Item, F: Unpin+Future<Output=Item>, MakeFuture: FnOnce() -> F> LazyFuture<Item, F, MakeFuture> {
    ///
    /// Creates a new lazy future. The function will be called when the future
    /// is required.
    ///
    pub fn new(make_future: MakeFuture) -> LazyFuture<Item, F, MakeFuture> {
        LazyFuture {
            core: Mutex::new(LazyFutureCore {
                make_future:    Some(make_future),
                the_future:     None
            })
        }
    }
}

impl<Item, F: Unpin+Future<Output=Item>, MakeFuture: FnOnce() -> F> Future for LazyFuture<Item, F, MakeFuture> {
    type Output = Item;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Item> {
        let mut core = self.core.lock().unwrap();

        if let Some(ref mut future) = core.the_future.as_mut() {
            // Just poll the future if it's set up
            return future.poll_unpin(context);
        }

        // Create a new future if it's not
        let make_future = core.make_future.take().unwrap();
        let future      = make_future();

        core.the_future = Some(future);
        core.the_future.as_mut().unwrap().poll_unpin(context)
   }
}
