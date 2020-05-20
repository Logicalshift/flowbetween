use super::core_element::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::traits::*;

use futures::prelude::*;

use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Performs a motion edit on this animation
    ///
    pub fn motion_edit<'a>(&'a mut self, motion_id: ElementId, motion_edit: &'a MotionEdit) -> impl 'a+Future<Output=()> {
        async move {
            use self::MotionEdit::*;

            // Convert the motion ID to an assigned element
            let motion_id = match motion_id.id() {
                Some(id)    => id,
                None        => { return; }
            };

            match motion_edit {
                Create                  => {
                    // Create the motion element
                    let motion          = Motion::None;
                    let motion          = MotionElement::new(ElementId::Assigned(motion_id), motion);
                    let motion          = Vector::Motion(motion);
                    let motion          = ElementWrapper::unattached_with_element(motion, Duration::from_millis(0));

                    // Write
                    self.request_one(StorageCommand::WriteElement(motion_id, motion.serialize_to_string())).await;
                }
                Delete                  => { self.request_one(StorageCommand::DeleteElement(motion_id)).await; }

                SetType(motion_type)    => { self.update_motion(motion_id, |mut motion| { motion.set_type(*motion_type); motion }).await; }
                SetOrigin(x, y)         => { self.update_motion(motion_id, |mut motion| { motion.set_origin((*x, *y)); motion }).await; }
                SetPath(time_curve)     => { self.update_motion(motion_id, |mut motion| { motion.set_path(time_curve.clone()); motion }).await; }
            }
        }
    }

    ///
    /// Updates an existing motion element
    ///
    fn update_motion<'a, UpdateFn>(&'a mut self, motion_id: i64, update_fn: UpdateFn) -> impl 'a+Future<Output=()>
    where UpdateFn: 'a+Send+Sync+Fn(Motion) -> Motion {
        async move {
            self.update_elements(vec![motion_id], |mut element_wrapper| {
                if let Vector::Motion(motion_element) = &element_wrapper.element {
                    let new_motion          = update_fn((*motion_element.motion()).clone());
                    let new_motion          = MotionElement::new(ElementId::Assigned(motion_id), new_motion);
                    let new_motion          = Vector::Motion(new_motion);
                    element_wrapper.element = new_motion;
                }

                ElementUpdate::ChangeWrapper(element_wrapper)
            }).await;
        }
    }
}
