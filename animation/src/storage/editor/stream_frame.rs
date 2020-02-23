use super::super::super::traits::*;

use flo_canvas::*;

use std::sync::*;
use std::time::{Duration};

///
/// A frame from a stream animation
///
pub struct StreamFrame {
    /// When this frame exists
    frame_time: Duration,
}

impl StreamFrame {
    ///
    /// Creates a new stream frame
    ///
    pub fn new(frame_time: Duration) -> StreamFrame {
        StreamFrame {
            frame_time: frame_time
        }
    }
}

impl Frame for StreamFrame {
    ///
    /// Time index of this frame relative to its keyframe
    ///
    fn time_index(&self) -> Duration {
        self.frame_time
    }

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut dyn GraphicsPrimitives) {
        unimplemented!()
    }

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        unimplemented!()
    }

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    ///
    fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>> {
        unimplemented!()
    }

    ///
    /// Retrieves a copy of the element with the specifed ID from this frame, if it exists
    ///
    fn element_with_id(&self, id: ElementId) -> Option<Vector> {
        unimplemented!()
    }

    ///
    /// Retrieves the IDs and types of the elements attached to the element with a particular ID
    ///
    /// (Element data can be retrieved via element_with_id)
    ///
    fn attached_elements(&self, id: ElementId) -> Vec<(ElementId, VectorType)> {
        unimplemented!()
    }
}
