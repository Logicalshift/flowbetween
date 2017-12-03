use super::vector_keyframe::*;
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
    id: u64,

    /// The key frames for this vector, in order
    keyframes: Vec<VectorKeyFrame>,

    /// The brush stroke that is currently being drawn
    active_brush_stroke: Option<BrushElement>
}

impl VectorLayer {
    ///
    /// Cretes a new vector layer
    /// 
    pub fn new(id: u64) -> VectorLayer {
        let core = VectorLayerCore {
            id:                     id,
            keyframes:              vec![],
            active_brush_stroke:    None
        };

        VectorLayer { 
            core: RwLock::new(core)
        }
    }
}

impl VectorLayerCore {
    ///
    /// Sorts the keyframes in order
    /// 
    fn sort_key_frames(&mut self) {
        self.keyframes.sort_by(|a, b| a.start_time().cmp(&b.start_time()));
    }
}

//
// == PAINTLAYER ==
//

impl PaintLayer for VectorLayerCore {
    fn start_brush_stroke(&mut self, start_time: Duration, initial_pos: BrushPoint) {
        // Start a new brush stroke, at a time relative to 0
        let element = BrushElement::new(start_time, initial_pos);

        self.active_brush_stroke = Some(element);
    }

    fn continue_brush_stroke(&mut self, point: BrushPoint) {
        // Add points to the active brush stroke
        if let Some(ref mut brush_stroke) = self.active_brush_stroke {
            brush_stroke.add_point(point);
        }
    }

    fn finish_brush_stroke(&mut self) {

    }

    fn cancel_brush_stroke(&mut self) {
        self.active_brush_stroke = None;
    }

    fn draw_current_brush_stroke(&self, gc: &mut GraphicsContext) {
        // Just pass the buck to the current brush stroke
        if let Some(ref brush_stroke) = self.active_brush_stroke {
            brush_stroke.render(gc);
        }
    }
}

//
// == KEYFRAMELAYER ==
//

impl KeyFrameLayer for VectorLayerCore {
    fn add_key_frame(&mut self, time_offset: Duration) {
        // TODO: do nothing if the keyframe is already created

        // Generate a new keyframe
        let new_keyframe = VectorKeyFrame::new(time_offset);

        // Add in order to the existing keyframes
        self.keyframes.push(new_keyframe);
        self.sort_key_frames();
    }

    fn move_key_frame(&mut self, from: Duration, to: Duration) {

    }
}

//
// == EDITING VIEWS ==
//

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
