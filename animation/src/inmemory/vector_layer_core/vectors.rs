use super::*;

impl VectorLayer for VectorLayerCore {
    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, when: Duration, new_element: Box<VectorElement>) {
        if let Some(keyframe) = self.find_nearest_keyframe(when){
            let when = when - keyframe.start_time();

            keyframe.add_element(when, new_element);
        }
    }
}
