use super::*;
use super::flo_store::*;
use super::layer_cache::*;
use super::vector_frame::*;
use super::super::result::Result;

use std::ops::{Range, Deref};
use std::time::Duration;
use std::collections::HashMap;

///
/// Represents a vector layer in a SQLite database
/// 
#[derive(Clone)]
pub struct SqliteVectorLayer<TFile: Unpin+FloFile+Send> {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The name of this layer, if it has one
    name: Option<String>,

    /// The currently active brush for this layer (or none if we need to fetch this from the database)
    /// The active brush is the brush most recently added to the keyframe at the specified point in time
    active_brush: Option<(Duration, Arc<dyn Brush>)>,

    /// Known layer caches for this layer
    frame_caches: Arc<Desync<HashMap<Duration, Weak<LayerCanvasCache<TFile>>>>>,

    /// Database core
    core: Arc<Desync<AnimationDbCore<TFile>>>
}

impl AnimationDb {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn get_layer_with_id(&self, assigned_id: u64) -> Option<SqliteVectorLayer<FloSqlite>> {
        SqliteVectorLayer::from_assigned_id(&self.core, assigned_id)
    }
}

impl<TFile: FloFile+Unpin+Send+'static> SqliteVectorLayer<TFile> {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn from_assigned_id(core: &Arc<Desync<AnimationDbCore<TFile>>>, assigned_id: u64) -> Option<SqliteVectorLayer<TFile>> {
        // Query for the 'real' layer ID
        let layer = core.sync(|core| {
            // Fetch the layer data (we need the 'real' ID here)
            core.db.query_layer_id_for_assigned_id(assigned_id)
        });

        // If the layer exists, create a SqliteVectorLayer
        layer.ok()
            .map(|(layer_id, name)| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    name:           name,
                    layer_id:       layer_id,
                    active_brush:   None,
                    core:           Arc::clone(core),
                    frame_caches:   Arc::new(Desync::new(HashMap::new()))
                }
            })
    }
}

impl<TFile: FloFile+Unpin+Send+'static> Layer for SqliteVectorLayer<TFile> {
    fn id(&self) -> u64 {
        self.assigned_id
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![LayerEditType::Vector]
    }

    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<dyn Iterator<Item=Duration>> {
        let from        = when.start;
        let until       = when.end;

        let keyframes   = self.core.sync(|core| core.db.query_key_frame_times_for_layer_id(self.layer_id, from, until));

        // Turn into an iterator
        let keyframes   = keyframes.unwrap_or_else(|_: SqliteAnimationError| vec![]);
        let keyframes   = Box::new(keyframes.into_iter());

        keyframes
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Box<dyn 'a+Deref<Target=dyn 'a+VectorLayer>>> {
        let vector_layer = self as &dyn VectorLayer;

        Some(Box::new(vector_layer))
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<dyn Frame> {
        let core: Result<Arc<dyn Frame>>    = self.core.sync(|core| {
            // TODO: this call is returning a 'QueryReturnedNoRows' error sometimes (which isn't too helpful as we don't know what's failing)
            let frame                       = VectorFrame::frame_at_time(&mut core.db, self.layer_id, time_index)?;
            let frame: Arc<dyn Frame>       = Arc::new(frame);

            Ok(frame)
        });

        core.unwrap()
    }

    fn previous_and_next_key_frame(&self, when: Duration) -> (Option<Duration>, Option<Duration>) {
        // Get the previous, next keyframes from the core
        let (previous, next) = self.core.sync(|core| core.db.query_previous_and_next_key_frame(self.layer_id, when)).unwrap();

        // Just want the durations and not the frame IDs here
        (previous.map(|(_, when)| when), next.map(|(_, when)| when))
    }

    fn get_canvas_cache_at_time(&self, time_index: Duration) -> Arc<dyn CanvasCache> {
        if let Some(layer_cache) = self.frame_caches.sync(|caches| caches.get(&time_index).and_then(|weak| weak.upgrade())) {
            // Use the existing layer cache if there is one
            layer_cache
        } else {
            // Create a new layer cache
            let layer_cache = LayerCanvasCache::cache_with_time(Arc::clone(&self.core), self.layer_id, time_index);
            let layer_cache = Arc::new(layer_cache);

            // Store so we can re-use the cache object later on
            let weak_cache = Arc::downgrade(&layer_cache);
            self.frame_caches.desync(move |caches| { caches.insert(time_index, weak_cache); });

            layer_cache
        }
    }
}

impl<TFile: FloFile+Unpin+Send+'static> VectorLayer for SqliteVectorLayer<TFile> {
    fn active_brush(&self, when: Duration) -> Option<Arc<dyn Brush>> {
        let layer_id = self.layer_id;
        self.core.sync(|core| core.get_active_brush_for_layer(layer_id, when))
    }
}
