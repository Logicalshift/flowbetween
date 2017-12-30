mod keyframes;
mod paint;

use super::super::brushes::*;
use super::super::traits::*;
use super::vector_keyframe::*;

use canvas::*;

use std::mem;
use std::sync::*;
use std::time::Duration;

///
/// The core of the vector layer
/// 
pub struct VectorLayerCore {
    // The ID assigned to this layer
    id: u64,

    /// The key frames for this vector, in order
    keyframes: Vec<Arc<VectorKeyFrame>>,

    /// The currently selected brush
    current_brush: Arc<Brush>,

    /// The brush stroke that is currently being drawn
    active_brush_stroke: Option<BrushElement>
}

impl VectorLayerCore {
    ///
    /// Creates a new vector layer core
    /// 
    pub fn new(id: u64) -> VectorLayerCore {
        VectorLayerCore {
            id:                     id,
            keyframes:              vec![],
            current_brush:          Arc::new(InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw)),
            active_brush_stroke:    None
        }
    }

    ///
    /// Returns the ID for this layer
    /// 
    pub fn id(&self) -> u64 {
        self.id
    }

    ///
    /// Sorts the keyframes in order
    /// 
    fn sort_key_frames(&mut self) {
        self.keyframes.sort_by(|a, b| a.start_time().cmp(&b.start_time()));
    }

    ///
    /// Finds the keyframe closest to the specified time
    /// 
    pub fn find_nearest_keyframe<'a>(&'a self, time: Duration) -> Option<&'a Arc<VectorKeyFrame>> {
        // Binary search for the key frame
        let search_result = self.keyframes.binary_search_by(|a| a.start_time().cmp(&time));

        match search_result {
            Ok(exact_frame)         => Some(&self.keyframes[exact_frame]),
            Err(following_frame)    => if following_frame == 0 {
                None
            } else {
                Some(&self.keyframes[following_frame-1])
            }
        }
    }
}
