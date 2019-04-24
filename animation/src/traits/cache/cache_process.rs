use futures::*;

///
/// Represents an item that is in the cache, or is in the process of being generated
///
pub enum CacheProcess<Result, Process: Future<Item=Result>+Send> {
    /// The item was already cached and is retrieved
    Cached(Result),

    /// The item has not been cached and is being generated
    Process(Process),
}

impl<Result: Clone, Process: Future<Item=Result>+Send> Future for CacheProcess<Result, Process> {
    type Item   = Result;
    type Error  = Process::Error;

    fn poll(&mut self) -> Poll<Result, Process::Error> {
        match self {
            CacheProcess::Cached(result)    => Ok(Async::Ready(result.clone())),
            CacheProcess::Process(process)  => process.poll(),
        }
    }
}
