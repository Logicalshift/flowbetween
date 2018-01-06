use super::super::super::traits::*;

use std::mem;
use std::sync::*;
use std::ops::Range;

// TODO: create this directly from the animation trait rather than
// via the layer (the current implementation requires weak refs and
// other crazy things)

///
/// Supplies the log editor for the vector layer
/// 
pub struct VectorLayerEditLog {
    layer_id: u64,

    /// The animation edit log (where final edits are comitted)
    animation_log: Arc<RwLock<PendingEditLog<AnimationEdit>>>,

    /// The edits pending for this edit log
    pending_edits: Vec<LayerEdit>
}

impl VectorLayerEditLog {
    ///
    /// Creates a new vector layer edit log
    /// 
    pub fn new(layer_id: u64, animation_log: Arc<RwLock<PendingEditLog<AnimationEdit>>>) -> VectorLayerEditLog {
        VectorLayerEditLog {
            layer_id:       layer_id,
            animation_log:  animation_log,
            pending_edits:  vec![]
        }
    }
}

impl PendingEditLog<LayerEdit> for VectorLayerEditLog {
    fn pending(&self) -> Vec<LayerEdit> {
        self.pending_edits.clone()
    }

    fn set_pending(&mut self, edits: &[LayerEdit]) {
        // Collect the edits into a vector
        self.pending_edits = edits.iter()
            .map(|edit| edit.clone())
            .collect();
    }

    fn commit_pending(&mut self) -> Range<usize> {
        // Fetch the pending edit list from this object
        let mut pending_edits = vec![];
        mem::swap(&mut pending_edits, &mut self.pending_edits);

        // Commit to the animation edit log
        // The animation should call the layer back to commit the actual edits
        let pending_edits: Vec<AnimationEdit> = pending_edits.into_iter()
            .map(|layer_edit| AnimationEdit::Layer(self.layer_id, layer_edit))
            .collect();

        // This will clobber any existing pending edits in the animation log
        let mut animation_log = self.animation_log.write().unwrap();
        animation_log.set_pending(&pending_edits);
        animation_log.commit_pending()
    }

    fn cancel_pending(&mut self) {
        self.pending_edits = vec![];
    }
}