use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::vector_frame::*;

use animation::brushes::*;

use rusqlite::*;
use std::ops::{Range, Deref};
use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
#[derive(Clone)]
pub struct SqliteVectorLayer<TFile: FloFile+Send> {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The currently active brush for this layer (or none if we need to fetch this from the database)
    /// The active brush is the brush most recently added to the keyframe at the specified point in time
    active_brush: Option<(Duration, Arc<Brush>)>,

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

impl<TFile: FloFile+Send+'static> SqliteVectorLayer<TFile> {
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
            .map(|layer_id| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    layer_id:       layer_id,
                    active_brush:   None,
                    core:           Arc::clone(core)
                }
            })
    }
}

impl<TFile: FloFile+Send+'static> Layer for SqliteVectorLayer<TFile> {
    fn id(&self) -> u64 {
        self.assigned_id
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![LayerEditType::Vector]
    }

    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<Iterator<Item=Duration>> {
        let from        = when.start;
        let until       = when.end;

        let keyframes   = self.core.sync(|core| core.db.query_key_frame_times_for_layer_id(self.layer_id, from, until));

        // Turn into an iterator
        let keyframes   = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes   = Box::new(keyframes.into_iter());

        keyframes
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Box<'a+Deref<Target='a+VectorLayer>>> {
        let vector_layer = self as &VectorLayer;

        Some(Box::new(vector_layer))
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        let core: Result<Arc<Frame>> = self.core.sync(|core| {
            let frame               = VectorFrame::frame_at_time(&mut core.db, self.layer_id, time_index)?;
            let frame: Arc<Frame>   = Arc::new(frame);

            Ok(frame)
        });

        core.unwrap()
    }
}

impl<TFile: FloFile+Send+'static> VectorLayer for SqliteVectorLayer<TFile> {
    fn active_brush(&self, when: Duration) -> Arc<Brush> {
        let layer_id = self.layer_id;
        self.core.sync(|core| core.get_active_brush_for_layer(layer_id, when))
    }
}
