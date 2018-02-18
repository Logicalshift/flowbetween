use futures::*;

use std::sync::*;

struct LazyFutureCore<Item, Error, F: Future<Item=Item, Error=Error>, MakeFuture: FnOnce() -> F> {
    /// Function to create a future
    make_future: Option<MakeFuture>,

    /// The future if it is real
    the_future: Option<F>
}

///
/// A lazy future creates an 'actual' future when polled but otherwise creates nothing
/// 
pub struct LazyFuture<Item, Error, F: Future<Item=Item, Error=Error>, MakeFuture: FnOnce() -> F> {
    core: Mutex<LazyFutureCore<Item, Error, F, MakeFuture>>
}

impl<Item, Error, F: Future<Item=Item, Error=Error>, MakeFuture: FnOnce() -> F> LazyFuture<Item, Error, F, MakeFuture> {
    ///
    /// Creates a new lazy future. The function will be called when the future
    /// is required.
    /// 
    pub fn new(make_future: MakeFuture) -> LazyFuture<Item, Error, F, MakeFuture> {
        LazyFuture {
            core: Mutex::new(LazyFutureCore {
                make_future:    Some(make_future),
                the_future:     None
            })
        }
    }
}

impl<Item, Error, F: Future<Item=Item, Error=Error>, MakeFuture: FnOnce() -> F> Future for LazyFuture<Item, Error, F, MakeFuture> {
    type Item = Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Item, Error> {
        let mut core = self.core.lock().unwrap();

        if let Some(ref mut future) = core.the_future.as_mut() {
            // Just poll the future if it's set up
            return future.poll();
        }
       
        // Create a new future if it's not
        let make_future = core.make_future.take().unwrap();
        let future      = make_future();

        core.the_future = Some(future);
        core.the_future.as_mut().unwrap().poll()
   }
}
