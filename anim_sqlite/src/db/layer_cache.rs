use super::flo_store::*;
use super::animation_core::*;

use desync::*;
use flo_canvas::*;
use flo_animation::*;

use futures::*;

use std::sync::*;
use std::time::Duration;

///
/// Canvas cache associated with a point in time
///
pub struct LayerCanvasCache<TFile: FloFile+Send> {
    /// Database core
    core: Arc<Desync<AnimationDbCore<TFile>>>,

    /// The time where we're retrieving (or storing) cached items
    when: Duration,

    /// The ID of the layer that we're retrieving/storing cached items
    layer_id: i64
}

impl<TFile: FloFile+Send> LayerCanvasCache<TFile> {
    ///
    /// Creates a layer cache at the specified time on a particular layer
    ///
    pub fn cache_with_time(db: Arc<Desync<AnimationDbCore<TFile>>>, layer_id: i64, when: Duration) -> LayerCanvasCache<TFile> {
        LayerCanvasCache {
            core:       db,
            when:       when,
            layer_id:   layer_id
        }
    }
}

impl<TFile: 'static+FloFile+Send> CanvasCache for LayerCanvasCache<TFile> {
    ///
    /// Invalidates any stored canvas with the specified type
    ///
    fn invalidate(&self, cache_type: CacheType) {
        // Copy the properties from this structure
        let layer_id    = self.layer_id;
        let when        = self.when;

        // Perform the update in the background
        self.core.desync(move |core| {
            // Remove the cached item
            let result = core.db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PopDeleteLayerCache(when, cache_type)
            ]);

            // Note any failures
            if let Err(result) = result {
                core.failure = Some(result.into())
            }
        });
    }

    ///
    /// Stores a particular drawing in the cache
    ///
    fn store(&self, cache_type: CacheType, items: Arc<Vec<Draw>>) {
        // Serialize the drawing instructions
        let mut draw_string = String::new();
        items.iter().for_each(|item| { item.encode_canvas(&mut draw_string); });

        // Copy the properties from this structure
        let layer_id    = self.layer_id;
        let when        = self.when;

        // Perform the update in the background
        self.core.desync(move |core| {
            // Store the cached item
            let result = core.db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PopStoreLayerCache(when, cache_type, draw_string)
            ]);

            // Note any failures
            if let Err(result) = result {
                core.failure = Some(result.into())
            }
        });
    }

    ///
    /// Retrieves the cached item at the specified time, if it exists
    ///
    fn retrieve(&self, cache_type: CacheType) -> Option<Arc<Vec<Draw>>> {
        self.core.sync(|core| core.db.query_layer_cached_drawing(self.layer_id, cache_type, self.when))
            .unwrap()
            .map(|drawing| Arc::new(drawing))
    }

    ///
    /// Retrieves the cached item, or calls the supplied function to generate it if it's not already in the cache
    ///
    fn retrieve_or_generate(&self, cache_type: CacheType, generate: Box<dyn Fn() -> Arc<Vec<Draw>> + Send>) -> CacheProcess<Arc<Vec<Draw>>, Box<dyn Future<Item=Arc<Vec<Draw>>, Error=Canceled>+Send>> {
        if let Some(result) = self.retrieve(cache_type) {
            // Cached data is already available
            CacheProcess::Cached(result)
        } else {
            // Process in the background using the cache work queue
            let core            = Arc::clone(&self.core);
            let work            = self.core.sync(|core| Arc::clone(&core.cache_work));
            let layer_id        = self.layer_id;
            let when            = self.when;

            // Note that as all caching work is done sequentially, it's not possible to accidentally call the generation function twice
            let future_result   = work.future(move |_| {
                // Attempt to retrieve an existing entry for this cached item
                let existing = core.sync(|core| core.db.query_layer_cached_drawing(layer_id, cache_type, when))
                    .unwrap()
                    .map(|drawing| Arc::new(drawing));

                if let Some(existing) = existing {
                    // Re-use the existing cached element if one has been generated in the meantime
                    existing
                } else {
                    // Call the generation function to create a new cache
                    let new_drawing = generate();

                    // Encode for storing in the database
                    let mut draw_string = String::new();
                    new_drawing.iter().for_each(|item| { item.encode_canvas(&mut draw_string); });

                    core.desync(move |core| {
                        // Store the cached item
                        let result = core.db.update(vec![
                            DatabaseUpdate::PushLayerId(layer_id),
                            DatabaseUpdate::PopStoreLayerCache(when, cache_type, draw_string)
                        ]);

                        // Note any failures
                        if let Err(result) = result {
                            core.failure = Some(result.into())
                        }
                    });

                    // The generated drawing is the cache result
                    new_drawing
                }
            });

            CacheProcess::Process(Box::new(future_result))
        }
    }
}
