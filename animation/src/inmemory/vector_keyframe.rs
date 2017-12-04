use super::super::traits::*;
use super::super::deref_map::*;

use std::time::Duration;
use std::ops::Deref;
use std::sync::*;

///
/// A keyframe in a vector layer
/// 
pub struct VectorKeyFrame {
    core: RwLock<VectorKeyFrameCore>
}

impl VectorKeyFrame {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration) -> VectorKeyFrame {
        VectorKeyFrame {
            core: RwLock::new(VectorKeyFrameCore::new(start_time))
        }
    }

    ///
    /// The start time of this key frame
    /// 
    pub fn start_time(&self) -> Duration {
        self.core.read().unwrap().start_time()
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    pub fn add_element(&self, new_element: Vector) {
        self.core.write().unwrap().add_element(new_element);
    }

    ///
    /// Retrieves the elements in this keyframe
    /// 
    pub fn elements<'a>(&'a self) -> Box<'a+Deref<Target=Vec<Vector>>> {
        let core            = self.core.read().unwrap();

        let elements = DerefMap::map(core, |core| &core.elements);

        Box::new(elements)
    }
}

///
/// Data storage for a vector keyframe
/// 
struct VectorKeyFrameCore {
    /// When this frame starts
    start_time: Duration,

    /// The elements in this key frame (ordered from back to front)
    elements: Vec<Vector>
}

impl VectorKeyFrameCore {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration) -> VectorKeyFrameCore {
        VectorKeyFrameCore {
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
}
