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
        let mut properties  = Arc::new(VectorProperties::default());

        self.keyframe.elements().iter().for_each(move |&(appearance_time, ref element)| {
            // Properties always update regardless of the time they're at (so the display is consistent)
            properties = element.update_properties(Arc::clone(&properties));

            if appearance_time <= offset {
                element.render(gc, &properties);
            }
        })
    }

    fn vector_elements<'a>(&'a self) -> Option<Box<'a+Iterator<Item=Vector>>> {
        let offset              = self.offset;
        let elements: Vec<_>    = self.keyframe.elements().iter()
            .filter(move |&&(appearance_time, _)| appearance_time <= offset)
            .map(|&(_, ref element)| element.clone())
            .collect();

        Some(Box::new(elements.into_iter()))
    }

    fn active_brush(&self) -> Option<(BrushDefinition, BrushDrawingStyle)> {
        let mut properties  = Arc::new(VectorProperties::default());

        self.keyframe.elements().iter()
            .for_each(|&(_appearance_time, ref element)| {
                properties = element.update_properties(Arc::clone(&properties));
            });

        Some(properties.brush.to_definition())
    }

    fn active_brush_properties(&self) -> Option<BrushProperties> {
        let mut properties  = Arc::new(VectorProperties::default());

        self.keyframe.elements().iter()
            .for_each(|&(_appearance_time, ref element)| {
                properties = element.update_properties(Arc::clone(&properties));
            });

        Some(properties.brush_properties)
    }
}
