use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::traits::*;

use flo_canvas::*;
use flo_stream::*;

use ::desync::*;
use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;
use std::time::{Duration};

///
/// Layer cache for the stream animation
///
pub struct StreamLayerCache {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore>>,

    /// The ID of the layer this is a cache for
    layer_id: u64,

    /// The time that this cache is for
    when: Duration,

    /// Used for scheduling generation operations
    generator: Arc<Desync<()>>
}

impl StreamLayerCache {
    ///
    /// Creates a new stream layer cache
    ///
    pub (super) fn new(core: Arc<Desync<StreamAnimationCore>>, layer_id: u64, when: Duration) -> StreamLayerCache {
        StreamLayerCache {
            core:       core,
            layer_id:   layer_id,
            when:       when,
            generator:  Arc::new(Desync::new(()))
        }
    }
}

impl CanvasCache for StreamLayerCache {
    ///
    /// Invalidates any stored canvas with the specified type
    ///
    fn invalidate(&self, cache_type: CacheType) {
        // Gather information
        let when        = self.when;
        let layer_id    = self.layer_id;
        let mut key     = String::new();
        cache_type.serialize(&mut key);

        // Ask the core to delete the cached value
        let _       = self.core.future(move |core| {
            async move {
                core.storage_requests.publish(vec![StorageCommand::DeleteLayerCache(layer_id, when, key)]).await;
                core.storage_responses.next().await;
            }.boxed()
        });
    }

    ///
    /// Stores a particular drawing in the cache
    ///
    fn store(&self, cache_type: CacheType, items: Arc<Vec<Draw>>) {
        // Gather information
        let when            = self.when;
        let layer_id        = self.layer_id;
        let mut key         = String::new();
        cache_type.serialize(&mut key);

        // Serialize the items
        let mut drawing     = String::new();
        items.encode_canvas(&mut drawing);

        // Ask the core to store the cached value
        let _               = self.core.future(move |core| {
            async move {
                core.storage_requests.publish(vec![StorageCommand::WriteLayerCache(layer_id, when, key, drawing)]).await;
                core.storage_responses.next().await;
            }.boxed()
        });
    }

    ///
    /// Retrieves the cached item at the specified time, if it exists
    ///
    fn retrieve(&self, cache_type: CacheType) -> Option<Arc<Vec<Draw>>> {
        // Gather information
        let when        = self.when;
        let layer_id    = self.layer_id;
        let mut key     = String::new();
        cache_type.serialize(&mut key);

        // Retrieve the value via a desync
        let core        = Arc::clone(&self.core);
        let value       = Desync::new(None);
        let _           = value.future(move |value| { 
            async move {
                // Ask the core for the cache value
                let response = core.future(move |core| {
                    async move {
                        core.storage_requests.publish(vec![StorageCommand::ReadLayerCache(layer_id, when, key)]).await;
                        core.storage_responses.next().await
                    }.boxed()
                }).await.unwrap();

                // Check the responses to generate the value
                *value = response
                    .and_then(|mut response| response.pop())
                    .and_then(|response| {
                        match response {
                            StorageResponse::LayerCache(cache_value)    => Some(cache_value),
                            _                                           => None
                        }
                    });
            }.boxed()
        });

        // Retrieve the value returned from the core
        let value       = value.sync(|value| value.take());

        // Try to deserialize as a canvas
        let value       = value.and_then(|value| decode_drawing(value.chars()).collect::<Result<Vec<_>, _>>().ok());
        let value       = value.map(|value| Arc::new(value));

        value
    }

    ///
    /// Retrieves the cached item, or calls the supplied function to generate it if it's not already in the cache
    ///
    fn retrieve_or_generate(&self, cache_type: CacheType, generate: Box<dyn Fn() -> Arc<Vec<Draw>> + Send>) -> CacheProcess<Arc<Vec<Draw>>, BoxFuture<'static, Arc<Vec<Draw>>>> {
        if let Some(result) = self.retrieve(cache_type) {
            // Already exists in the cache
            CacheProcess::Cached(result)
        } else {
            // Gather information
            let when        = self.when;
            let layer_id    = self.layer_id;
            let mut key     = String::new();
            cache_type.serialize(&mut key);

            // Generate on the core
            let core        = Arc::clone(&self.core);
            let generator   = Arc::clone(&self.generator);

            CacheProcess::Process(async move {
                // Generate the drawing
                let drawing         = generator.future(move |_| async move { generate() }.boxed()).await.unwrap();

                // Serialize the drawing
                let mut serialized  = String::new();
                drawing.encode_canvas(&mut serialized);

                // Store using the core
                let _ = core.future(move |core| {
                    async move {
                        core.storage_requests.publish(vec![StorageCommand::WriteLayerCache(layer_id, when, key, serialized)]).await;
                        core.storage_responses.next().await;
                    }.boxed()
                });

                // Return the drawing as the result
                drawing
            }.boxed())
        }
    }
}
