use super::super::traits::*;

use std::time::Duration;

///
/// Represents a keuyframe in a vector animation
/// 
pub struct VectorKeyFrame {
    /// When this frame starts
    start_time: Duration,

    /// The elements in this key frame (ordered from back to front)
    elements: Vec<Vector>
}

impl VectorKeyFrame {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration) -> VectorKeyFrame {
        VectorKeyFrame {
            start_time: start_time,
            elements:   vec![]
        }
    }

    ///
    /// The start time of this key frame
    /// 
    pub fn start_time(&self) -> Duration {
        self.start_time
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    pub fn add_element(&mut self, new_element: Vector) {
        self.elements.push(new_element);
    }

    ///
    /// Retrieves the elements in this keyframe
    /// 
    pub fn elements<'a>(&'a self) -> &'a Vec<Vector> {
        &self.elements
    }
}
