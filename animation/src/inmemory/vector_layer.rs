use super::vector_layer_core::*;
use super::super::traits::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector layer. Vector layers support brush and vector objects.
/// 
pub struct VectorLayer {
    /// The core of this layer
    core: RwLock<VectorLayerCore>
}

impl VectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64) -> VectorLayer {
        let core = VectorLayerCore::new(id);

        VectorLayer { 
            core: RwLock::new(core)
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

impl Editable<KeyFrameLayer+'static> for VectorLayer {
    fn edit(&self) -> Option<Editor<KeyFrameLayer+'static>> { 
        let core: &RwLock<KeyFrameLayer> = &self.core;
        Some(Editor::new(core.write().unwrap())) 
    }

    fn read(&self) -> Option<Reader<KeyFrameLayer+'static>> { 
        let core: &RwLock<KeyFrameLayer> = &self.core;
        Some(Reader::new(core.read().unwrap())) 
    }
}

impl Layer for VectorLayer {
    fn id(&self) -> u64 {
        self.core.read().unwrap().id()
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Box<Frame> {
        unimplemented!()
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        unimplemented!()
    }
}
