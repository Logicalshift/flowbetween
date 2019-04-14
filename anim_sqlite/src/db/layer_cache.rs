use super::flo_store::*;
use super::animation_core::*;

use desync::*;
use flo_canvas::*;
use flo_animation::*;

use std::sync::*;
use std::time::Duration;

///
/// Canvas cache associated with a point in time 
///
pub struct LayerCanvasCache<TFile: FloFile+Send> {
    /// Database core
    core: Arc<Desync<AnimationDbCore<TFile>>>
}

impl<TFile: FloFile+Send> LayerCanvasCache<TFile> {
    ///
    /// Creates a layer cache at the specified time on a particular layer
    ///
    pub fn cache_with_time(db: Arc<Desync<AnimationDbCore<TFile>>>, layer_id: i64, when: Duration) -> LayerCanvasCache<TFile> {
        LayerCanvasCache {
            core: db
        }
    }
}

impl<TFile: FloFile+Send> CanvasCache for LayerCanvasCache<TFile> {
    ///
    /// Invalidates any stored canvas with the specified type
    ///
    fn invalidate(&self, cache_type: CacheType) {
        unimplemented!()
    }

    ///
    /// Stores a particular drawing in the cache
    ///
    fn store(&self, cache_type: CacheType, items: &mut dyn Iterator<Item=Draw>) {
        unimplemented!()
    }

    ///
    /// Retrieves the cached item at the specified time, if it exists
    ///
    fn retrieve(&self, cache_type: CacheType) -> Option<Vec<Draw>> {
        unimplemented!()
    }
}
