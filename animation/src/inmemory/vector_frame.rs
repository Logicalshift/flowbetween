use super::super::traits::*;
use super::vector_keyframe::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a ready-to-render vector frame
/// 
pub struct VectorFrame {
    /// The keyframe that will be rendered
    keyframe: Arc<VectorKeyFrame>,

    /// The offset into the frame that this should render
    offset: Duration,
}

impl VectorFrame {
    ///
    /// Creates a new vector keyframe
    /// 
    pub fn new(keyframe: Arc<VectorKeyFrame>, offset: Duration) -> VectorFrame {
        VectorFrame {
            keyframe:   keyframe,
            offset:     offset
        }
    }
}

impl Frame for VectorFrame {
    fn time_index(&self) -> Duration {
        self.keyframe.start_time() + self.offset
    }

    fn render_to(&self, gc: &mut GraphicsPrimitives) {
        let offset          = self.offset;
        let mut properties  = VectorProperties::default();

        self.keyframe.elements().iter().for_each(move |&(appearance_time, ref element)| {
            // Properties always update regardless of the time they're at (so the display is consistent)
            element.update_properties(&mut properties);

            if appearance_time <= offset {
                element.render(gc, &properties);
            }
        })
    }
}