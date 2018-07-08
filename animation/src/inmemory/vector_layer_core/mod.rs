mod vectors;
mod edit;

pub use self::edit::*;

use super::super::traits::*;
use super::vector_map::*;
use super::vector_keyframe::*;

use std::sync::*;
use std::time::Duration;

///
/// The core of the vector layer
/// 
pub struct VectorLayerCore {
    // The ID assigned to this layer
    id: u64,

    /// The vector map for this layer
    vector_map: VectorMap,

    /// The key frames for this vector, in order
    keyframes: Vec<Arc<VectorKeyFrame>>,
}

impl VectorLayerCore {
    ///
    /// Creates a new vector layer core
    /// 
    pub fn new(id: u64) -> VectorLayerCore {
        VectorLayerCore {
            id:                     id,
            vector_map:             VectorMap::new(),
            keyframes:              vec![],
        }
    }

    ///
    /// Returns the ID for this layer
    /// 
    pub fn id(&self) -> u64 {
        self.id
    }

    ///
    /// Retrieves the keyframes in this layer
    /// 
    pub fn keyframes<'a>(&'a self) -> impl Iterator<Item=Arc<VectorKeyFrame>>+'a {
        self.keyframes.iter().cloned()
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
