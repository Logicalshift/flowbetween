mod vectors;

use super::super::traits::*;
use super::vector_keyframe::*;

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
}

impl VectorLayerCore {
    ///
    /// Creates a new vector layer core
    /// 
    pub fn new(id: u64) -> VectorLayerCore {
        VectorLayerCore {
            id:                     id,
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

    ///
    /// Adds a new key frame to this core 
    /// 
    pub fn add_key_frame(&mut self, time_offset: Duration) {
        // TODO: do nothing if the keyframe is already created

        // Generate a new keyframe
        let new_keyframe = VectorKeyFrame::new(time_offset);

        // Add in order to the existing keyframes
        self.keyframes.push(Arc::new(new_keyframe));
        self.sort_key_frames();
    }

    ///
    /// Removes a keyframe from this core
    /// 
    pub fn remove_key_frame(&mut self, time_offset: Duration) {
        // Binary search for the key frame
        let search_result = self.keyframes.binary_search_by(|a| a.start_time().cmp(&time_offset));

        // Remove only if we found an exact match
        if let Ok(frame_number) = search_result {
            self.keyframes.remove(frame_number);
        }
    }

    ///
    /// Applies an edit command to this layer
    /// 
    pub fn apply_edit(&mut self, edit: &LayerEdit) {
        use LayerEdit::*;

        match edit {
            &Paint(_, _)                        => unimplemented!(),
            &AddKeyFrame(ref time_offset)       => self.add_key_frame(*time_offset),
            &RemoveKeyFrame(ref time_offset)    => self.remove_key_frame(*time_offset)
        }
    }
}
