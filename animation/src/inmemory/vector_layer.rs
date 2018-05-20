use super::empty_frame::*;
use super::vector_frame::*;
use super::vector_layer_core::*;
use super::super::traits::*;

use std::sync::*;
use std::ops::Range;
use std::time::Duration;

///
/// Represents a vector layer. Vector layers support brush and vector objects.
/// 
pub struct InMemoryVectorLayer {
    /// The core data for this layer
    core: Mutex<VectorLayerCore>
}

impl InMemoryVectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64) -> InMemoryVectorLayer {
        let core = VectorLayerCore::new(id);

        InMemoryVectorLayer { 
            core:       Mutex::new(core)
        }
    }

    ///
    /// Edits this layer
    /// 
    #[inline]
    pub fn edit(&self, edit: LayerEdit) {
        self.core.lock().unwrap().edit(edit);
    }

    ///
    /// Performs an edit on an element contained within this animation
    /// 
    pub fn edit_element(&self, element_id: ElementId, when: Duration, edit: &ElementEdit) {
        // TODO
    }
}

impl Layer for InMemoryVectorLayer {
    fn id(&self) -> u64 {
        self.core.lock().unwrap().id()
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        let core = self.core.lock().unwrap();

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

    fn get_key_frames_during_time(&self, _when: Range<Duration>) -> Box<Iterator<Item=Duration>> {
        unimplemented!()
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        return vec![
            LayerEditType::Vector
        ];
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>> {
        let core: &Mutex<VectorLayer> = &self.core;

        Some(Reader::new(core.lock().unwrap()))
    }
}
