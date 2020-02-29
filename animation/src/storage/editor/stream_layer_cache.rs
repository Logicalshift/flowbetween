use super::stream_animation_core::*;
use super::super::storage_api::*;
use super::super::super::traits::*;

use flo_canvas::*;

use ::desync::*;
use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;
use std::time::{Duration};

///
/// Layer cache for the 
///
pub struct StreamLayerCache {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore>>,

    /// The ID of the layer this is a cache for
    layer_id: u64,

    /// The time that this
    when: Duration
}

impl StreamLayerCache {
    ///
    /// Creates a new stream layer cache
    ///
    pub (super) fn new(core: Arc<Desync<StreamAnimationCore>>, layer_id: u64, when: Duration) -> StreamLayerCache {
        StreamLayerCache {
            core:       core,
            layer_id:   layer_id,
            when:       when
        }
    }
}

impl CanvasCache for StreamLayerCache {
    ///
    /// Invalidates any stored canvas with the specified type
    ///
    fn invalidate(&self, cache_type: CacheType) {

    }

    ///
    /// Stores a particular drawing in the cache
    ///
    fn store(&self, cache_type: CacheType, items: Arc<Vec<Draw>>) {

    }

    ///
    /// Retrieves the cached item at the specified time, if it exists
    ///
    fn retrieve(&self, cache_type: CacheType) -> Option<Arc<Vec<Draw>>> {
        None
    }

    ///
    /// Retrieves the cached item, or calls the supplied function to generate it if it's not already in the cache
    ///
    fn retrieve_or_generate(&self, cache_type: CacheType, generate: Box<dyn Fn() -> Arc<Vec<Draw>> + Send>) -> CacheProcess<Arc<Vec<Draw>>, BoxFuture<'static, Arc<Vec<Draw>>>> {
        unimplemented!()
    }
}
