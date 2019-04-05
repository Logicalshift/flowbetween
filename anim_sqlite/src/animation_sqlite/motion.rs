use flo_animation::AnimationMotion;
use super::*;

use flo_animation::*;

use futures::*;
use std::ops::Range;
use std::time::Duration;

impl AnimationMotion for SqliteAnimation {
    fn assign_element_id(&self) -> ElementId {
        self.db.assign_element_id()
    }

    fn get_motion_ids(&self, when: Range<Duration>) -> Box<dyn Stream<Item=ElementId, Error=()>> {
        unimplemented!()
    }

    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        self.db.get_motion(motion_id)
    }

    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        self.db.get_motions_for_element(element_id)
    }

    fn get_elements_for_motion(&self, motion_id: ElementId) -> Vec<ElementId> {
        self.db.get_elements_for_motion(motion_id)
    }
}
