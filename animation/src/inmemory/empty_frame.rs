use super::super::traits::*;

use canvas::*;

use std::time::Duration;

///
/// A frame with nothing in it
/// 
pub struct EmptyFrame {
    time_index: Duration
}

impl EmptyFrame {
    ///
    /// Creates a new empty frame at a particular time index
    ///
    pub fn new(time_index: Duration) -> EmptyFrame {
        EmptyFrame {
            time_index: time_index
        }
    }
}

impl Frame for EmptyFrame {
    fn time_index(&self) -> Duration {
        self.time_index
    }

    fn render_to(&self, _gc: &mut dyn GraphicsPrimitives) {
    }

    fn vector_elements(&self) -> Option<Box<dyn Iterator<Item=Vector>>> {
        None
    }

    fn element_with_id<'a>(&'a self, _id: ElementId) -> Option<Vector> {
        None
    }

    fn active_brush(&self) -> Option<(BrushDefinition, BrushDrawingStyle)> { None }

    fn active_brush_properties(&self) -> Option<BrushProperties> { None }
}
