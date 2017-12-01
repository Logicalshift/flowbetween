use super::super::traits::*;

use ui::canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector layer. Vector layers support brush and vector objects.
/// 
pub struct VectorLayer {
    /// The core of this layer
    core: RwLock<VectorLayerCore>
}

struct VectorLayerCore {
    // The ID assigned to this layer
    id: u64
}

impl VectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64) -> VectorLayer {
        let core = VectorLayerCore {
            id: id
        };

        VectorLayer { 
            core: RwLock::new(core)
        }
    }
}

impl PaintLayer for VectorLayerCore {
    fn start_brush_stroke(&mut self, start_time: Duration, initial_pos: (f64, f64), pressure: f64) {

    }

    fn continue_brush_stroke(&mut self, next_point: (f64, f64), pressure: f64){

    }

    fn finish_brush_stroke(&mut self) {

    }

    fn cancel_brush_stroke(&mut self) {

    }

    fn draw_current_brush_stroke(&self, gc: &mut GraphicsContext) {

    }
}

impl KeyFrameLayer for VectorLayerCore {
    fn add_key_frame(&mut self, time_offset: Duration) {

    }

    fn move_key_frame(&mut self, from: Duration, to: Duration) {

    }
}

impl Editable<PaintLayer+'static> for VectorLayer {
    fn open(&self) -> Option<Editor<PaintLayer+'static>> {
        let core: &RwLock<PaintLayer> = &self.core;
        Some(Editor::new(core.write().unwrap())) 
    }

    fn read(&self) -> Option<Reader<PaintLayer+'static>> {
        let core: &RwLock<PaintLayer> = &self.core;
        Some(Reader::new(core.read().unwrap())) 
    }
}

impl Editable<KeyFrameLayer+'static> for VectorLayer {
    fn open(&self) -> Option<Editor<KeyFrameLayer+'static>> { 
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
        self.core.read().unwrap().id
    }

    fn get_frame_at_time<'a>(&self, time_index: Duration) -> &'a Frame {
        unimplemented!()
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        unimplemented!()
    }
}
