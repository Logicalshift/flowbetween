use super::empty_frame::*;
use super::vector_frame::*;
use super::vector_layer_core::*;
use super::super::traits::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector layer. Vector layers support brush and vector objects.
/// 
pub struct InMemoryVectorLayer {
    /// The core data for this layer
    core: RwLock<VectorLayerCore>
}

impl InMemoryVectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64) -> InMemoryVectorLayer {
        let core = VectorLayerCore::new(id);

        InMemoryVectorLayer { 
            core:       RwLock::new(core)
        }
    }

    ///
    /// Applies new edits for this layer
    /// 
    pub fn apply_edit(&self, edit: &LayerEdit) {
        let mut core = self.core.write().unwrap();

        core.apply_edit(edit);
    }
}

impl Layer for InMemoryVectorLayer {
    fn id(&self) -> u64 {
        self.core.read().unwrap().id()
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        let core = self.core.read().unwrap();

        // Look up the keyframe in the core
        let keyframe = core.find_nearest_keyframe(time_index);
        if let Some(keyframe) = keyframe {
            // Found a keyframe: return a vector frame from it
            Arc::new(VectorFrame::new(keyframe.clone(), time_index - keyframe.start_time()))
        } else {
            // No keyframe at this point in time
            Arc::new(EmptyFrame::new(time_index))
        }
    }

    fn add_key_frame(&mut self, when: Duration) {
        self.core.write().unwrap().add_key_frame(when);
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        unimplemented!()
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        unimplemented!()
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>> {
        unimplemented!()
    }

    fn edit_vectors<'a>(&'a mut self) -> Option<Editor<'a, VectorLayer>> {
        unimplemented!()
    }
}
