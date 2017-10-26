use super::draw::*;

use futures::*;

///
/// A canvas is an abstract interface for drawing graphics. It doesn't actually provide a means to
/// render anything, but rather a way to describe how things should be drawn and pass those on to
/// a renderer elsewhere. 
///
pub struct Canvas {
    /// What was drawn since the last clear command was sent to this canvas
    drawing_since_last_clear: Vec<Draw>
}

impl Canvas {
    ///
    /// Creates a new, blank, canvas
    ///
    pub fn new() -> Canvas {
        // A canvas is initially just a clear command
        Canvas { 
            drawing_since_last_clear: vec![ Draw::ClearCanvas ]
        }
    }

    ///
    /// Sends some new drawing commands to this canvas
    ///
    pub fn draw(&mut self, to_draw: &[Draw]) {
        unimplemented!()
    }
}
