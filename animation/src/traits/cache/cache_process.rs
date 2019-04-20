use futures::*;

use std::mem;

///
/// Represents an item that is in the cache, or is in the process of being generated
///
pub enum CacheProcess<Result, Process: Future<Item=Result>> {
    /// The item was already cached and is retrieved
    Cached(Result),

    /// The item has not been cached and is being generated
    Process(Process),

    /// The item was cached but has been consumed
    Consumed
}

impl<Result, Process: Future<Item=Result>> Future for CacheProcess<Result, Process> {
    type Item   = Result;
    type Error  = Process::Error;

    fn poll(&mut self) -> Poll<Result, Process::Error> {
        match self {
            CacheProcess::Cached(_)         => {
                let mut result = CacheProcess::Consumed;
                mem::swap(&mut result, self);

                match result {
                    CacheProcess::Cached(result)    => Ok(Async::Ready(result)),
                    _                               => panic!("Result vanished")
                }
            },
            CacheProcess::Process(process)  => process.poll(),
            CacheProcess::Consumed          => panic!("Cache process result has already been consumed")
        }
    }
}
