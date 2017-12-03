use super::*;

impl KeyFrameLayer for VectorLayerCore {
    fn add_key_frame(&mut self, time_offset: Duration) {
        // TODO: do nothing if the keyframe is already created

        // Generate a new keyframe
        let new_keyframe = VectorKeyFrame::new(time_offset);

        // Add in order to the existing keyframes
        self.keyframes.push(Arc::new(new_keyframe));
        self.sort_key_frames();
    }

    fn move_key_frame(&mut self, from: Duration, to: Duration) {
        unimplemented!()
    }

    fn remove_key_frame(&mut self, time_offset: Duration) {
        // Binary search for the key frame
        let search_result = self.keyframes.binary_search_by(|a| a.start_time().cmp(&time_offset));

        // Remove only if we found an exact match
        if let Ok(frame_number) = search_result {
            self.keyframes.remove(frame_number);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_add_key_frame() {
        let mut core = VectorLayerCore::new(0);

        assert!(core.keyframes.len() == 0);
        core.add_key_frame(Duration::from_millis(1000));

        assert!(core.keyframes.len() == 1);
        assert!(core.keyframes[0].start_time() == Duration::from_millis(1000));
    }

    #[test]
    fn can_remove_key_frame() {
        let mut core = VectorLayerCore::new(0);

        core.add_key_frame(Duration::from_millis(1000));
        core.add_key_frame(Duration::from_millis(2000));
        core.add_key_frame(Duration::from_millis(200));
        core.add_key_frame(Duration::from_millis(3000));

        assert!(core.keyframes.len() == 4);

        core.remove_key_frame(Duration::from_millis(1000));
        assert!(core.keyframes.len() == 3);

        assert!(core.keyframes[0].start_time() == Duration::from_millis(200));
        assert!(core.keyframes[1].start_time() == Duration::from_millis(2000));
        assert!(core.keyframes[2].start_time() == Duration::from_millis(3000));
    }

    #[test]
    fn inexact_time_does_not_remove_key_frame() {
        let mut core = VectorLayerCore::new(0);

        core.add_key_frame(Duration::from_millis(3000));
        core.add_key_frame(Duration::from_millis(200));
        core.add_key_frame(Duration::from_millis(1000));
        core.add_key_frame(Duration::from_millis(2000));

        assert!(core.keyframes.len() == 4);

        core.remove_key_frame(Duration::from_millis(1001));
        assert!(core.keyframes.len() == 4);
    }
}
