use super::*;

impl KeyFrameLayer for VectorLayerCore {
    fn add_key_frame(&mut self, time_offset: Duration) {
        // TODO: do nothing if the keyframe is already created

        // Generate a new keyframe
        let new_keyframe = VectorKeyFrame::new(time_offset);

        // Add in order to the existing keyframes
        self.keyframes.push(new_keyframe);
        self.sort_key_frames();
    }

    fn move_key_frame(&mut self, from: Duration, to: Duration) {
        unimplemented!()
    }

    fn remove_key_frame(&mut self, time_offset: Duration) {
        unimplemented!()
    }
}
