use super::empty_frame::*;
use super::vector_frame::*;
use super::vector_layer_core::*;
use super::super::traits::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector layer. Vector layers support brush and vector objects.
/// 
pub struct VectorLayer {
    /// The edit log where edits to this layer should be committed
    // edit_log: Weak<RwLock<MutableEditLog<AnimationEdit>>>,

    /// The core data for this layer
    core: RwLock<VectorLayerCore>
}

impl VectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64, edit_log: &Arc<RwLock<MutableEditLog<AnimationEdit>>>) -> VectorLayer {
        let core = VectorLayerCore::new(id);

        VectorLayer { 
            // edit_log:   Arc::downgrade(edit_log),
            core:       RwLock::new(core)
        }
    }

    ///
    /// Applies new edits for this layer
    /// 
    pub fn apply_new_edits(&self, edits: &[LayerEdit]) {
        let mut core = self.core.write().unwrap();

        for edit in edits {
            core.apply_edit(edit);
        }
    }
}

//
// == EDITING VIEWS ==
//

impl Editable<PaintLayer+'static> for VectorLayer {
    fn edit(&self) -> Option<Editor<PaintLayer+'static>> {
        let core: &RwLock<PaintLayer> = &self.core;
        Some(Editor::new(core.write().unwrap())) 
    }

    fn read(&self) -> Option<Reader<PaintLayer+'static>> {
        let core: &RwLock<PaintLayer> = &self.core;
        Some(Reader::new(core.read().unwrap())) 
    }
}

impl Editable<MutableEditLog<LayerEdit>+'static> for VectorLayer {
    fn edit(&self) -> Option<Editor<MutableEditLog<LayerEdit>+'static>> {
        /*
        self.edit_log.upgrade()
            .map(|edit_log| VectorLayerEditLog::new(edit_log))
            .map(|editor| Editor::new(Arc::new(editor)))
        */
        None
    }

    fn read(&self) -> Option<Reader<MutableEditLog<LayerEdit>+'static>> {
        /*
        self.edit_log.upgrade()
            .map(|edit_log| VectorLayerEditLog::new(edit_log))
            .map(|editor| Reader::new(Arc::new(editor)))
        */
        None
    }
}

impl Layer for VectorLayer {
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

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        unimplemented!()
    }
}
