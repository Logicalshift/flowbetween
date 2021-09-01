use crate::path::*;

use flo_canvas::*;

use std::time::{Duration};

///
/// Represents an animated layer of a vector drawing. This accepts commands in the form
/// of `Draw` instructions, although it will only render to a single layer in the finished
/// rendering: sprite and layer commands will be ignored.
///
#[derive(Clone)]
pub struct AnimationLayer {
    /// The current state of the layer drawing
    layer_state: LayerDrawingToPaths,

    /// The drawing that has been performed so far
    drawing: Vec<AnimationPath>
}

impl AnimationLayer {
    ///
    /// Clears this layer
    ///
    pub fn clear(&mut self) {
        self.drawing.clear();
        self.drawing.extend(self.layer_state.draw([Draw::ClearLayer]));
    }

    ///
    /// Sets the time that paths added to this layer should appear
    ///
    pub fn set_time(&mut self, drawing_time: Duration) {
        self.layer_state.set_time(drawing_time);
    }

    ///
    /// Adds a new path to this layer
    ///
    pub fn add_path(&mut self, path: AnimationPath) {
        self.drawing.push(path);
    }

    ///
    /// Adds drawing onto this layer
    ///
    pub fn draw<DrawIter: IntoIterator<Item=Draw>>(&mut self, drawing: DrawIter) {
        self.drawing.extend(self.layer_state.draw(drawing));
    }
}
