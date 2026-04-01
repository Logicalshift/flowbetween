use super::cache_type::*;
use super::cache_process::*;

use futures::future::{BoxFuture};

use flo_canvas::*;

use std::sync::*;

///
/// Trait provided by things that can cache and retrieve canvas drawing instructions
///
/// Elements and layers can cache their rendering instructions so that they can render more quickly. An example of where
/// this might be used is with onion skins: these are built up by adding together all the paths in the frame, which is
/// a slow operation (but which generates a shape that can be rendered quickly). Storing the results of this operation
/// in a cache ensures that omnion skins can be rendered quickly.
///
pub trait CanvasCache {
    ///
    /// Invalidates any stored canvas with the specified type
    ///
    fn invalidate(&self, cache_type: CacheType);

    ///
    /// Stores a particular drawing in the cache
    ///
    fn store(&self, cache_type: CacheType, items: Arc<Vec<Draw>>);

    ///
    /// Retrieves the cached item at the specified time, if it exists
    ///
    fn retrieve(&self, cache_type: CacheType) -> Option<Arc<Vec<Draw>>>;

    ///
    /// Retrieves the cached item, or calls the supplied function to generate it if it's not already in the cache
    ///
    fn retrieve_or_generate(&self, cache_type: CacheType, generate: Box<dyn Fn() -> Arc<Vec<Draw>> + Send>) -> CacheProcess<Arc<Vec<Draw>>, BoxFuture<'static, Arc<Vec<Draw>>>>;
}
