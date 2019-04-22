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
    fn store(&self, cache_type: CacheType, items: Box<dyn Iterator<Item=Draw>>) {
        // Serialize the drawing instructions
        let mut draw_string = String::new();
        items.for_each(|item| { item.encode_canvas(&mut draw_string); });

        // Copy the properties from this structure
        let layer_id    = self.layer_id;
        let when        = self.when;

        // Perform the update in the background
        self.core.desync(move |core| {
            // Remove the cached item
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
    fn retrieve(&self, cache_type: CacheType) -> Option<Vec<Draw>> {
        self.core.sync(|core| core.db.query_layer_cached_drawing(self.layer_id, cache_type, self.when))
            .unwrap()
    }

    ///
    /// Retrieves the cached item, or calls the supplied function to generate it if it's not already in the cache
    ///
    fn retrieve_or_generate(&self, generate: Box<dyn Fn() -> Vec<Draw> + Send>) -> CacheProcess<Vec<Draw>, Box<dyn Future<Item=Vec<Draw>, Error=()>>> {
        unimplemented!()
    }
}
