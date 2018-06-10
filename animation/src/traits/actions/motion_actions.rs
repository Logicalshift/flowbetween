use super::edit_action::*;
use super::super::edit::*;
use super::super::motion::*;
use super::super::animation::*;
use super::super::time_path::*;

use std::time::Duration;
use std::collections::{HashSet, HashMap};

///
/// Edit actions that cause objects to move
/// 
pub enum MotionEditAction {
    /// Moves a set of elements via a drag from a particular spot to another spot
    /// 
    /// This is essentially the action performed when a user drags an item from one place
    /// to another (the two coordinates are the start and end position of the drag).
    /// 
    /// For each element, the action depends on what motions the element is already performing.
    /// 
    /// If there's an existing translate motion, it's updated so that at the specified time,
    /// what was at the 'from' point is now at the 'to' point.
    /// 
    /// If there's no existing translate motion, one is created with the 'from' point as the
    /// origin and a 0s movement.
    /// 
    /// If a translation that is being updated is attached to an element outside of the set
    /// that is being changed, the attached translation is changed to a new ID.
    MoveElements(Vec<ElementId>, Duration, (f32, f32), (f32, f32))
}

impl EditAction for MotionEditAction {
    ///
    /// Converts this edit action into a set of animation edits for a particular animation
    /// 
    fn to_animation_edits<Anim: Animation>(&self, animation: &Anim) -> Vec<AnimationEdit> {
        use self::MotionEditAction::*;

        match self {
            MoveElements(elements, when, from, to)  => move_elements_edit(animation, elements, when, from, to)
        }
    }
}

///
/// Generates an edit for a set of elements that currently have no translate motion attached to them
/// that attaches a suitable motion that just translates them instantly at a point in time
/// 
fn static_move_edit<Anim: Animation>(animation: &Anim, elements: &HashSet<ElementId>, when: &Duration, from: &(f32, f32), to: &(f32, f32)) -> Vec<AnimationEdit> {
    if elements.len() > 0 {
        // Create a new motion, then attach it to the static elements
        let static_motion_id    = animation.motion().assign_motion_id();
        let target_point        = TimePoint::new(to.0, to.1, when.clone());
        
        // Creates a motion that instantaneously moves from the 'from' point to the 'to' point 
        let create_motion       = vec![
            MotionEdit::Create,
            MotionEdit::SetType(MotionType::Translate),
            MotionEdit::SetOrigin(from.0, from.1),
            MotionEdit::SetPath(TimeCurve::new(target_point, target_point))
        ];

        // Attach the static elements
        let attach_elements     = elements.iter()
            .map(|element_id| MotionEdit::Attach(*element_id));

        // Turn into a series of animation edits
        create_motion.into_iter()
            .chain(attach_elements)
            .map(|motion_edit| AnimationEdit::Motion(static_motion_id, motion_edit))
            .collect()
    } else {
        // No static elements = no static element translation
        vec![]
    }
}

///
/// Generates updates for elements attached to an existing motion
/// 
fn dynamic_move_edit<Anim: Animation>(animation: &Anim, motion_id: ElementId, elements: &Vec<ElementId>, when: &Duration, from: &(f32, f32), to: &(f32, f32)) -> Vec<AnimationEdit> {
    if let Some(Motion::Translate(translate)) = animation.motion().get_motion(motion_id) {
        // Set an existing point in this translate motion
        let when_millis         = to_millis(*when) as f32;
        let (move_x, move_y)    = (to.0-from.0, to.1-from.1);

        let existing_curve      = translate.translate;
        let origin              = translate.origin;
        let existing_point      = existing_curve.point_at_time(when_millis).unwrap_or_else(|| TimePoint(origin.0, origin.1, when_millis));
        let moved_point         = existing_point + TimePoint(move_x, move_y, 0.0);

        let updated_curve       = existing_curve.set_point_at_time(*when, (moved_point.0, moved_point.1));

        // Check if there are any elements that are attached to this motion but are not being moved
        let attached_to             = animation.motion().get_elements_for_motion(motion_id);
        let elements: HashSet<_>    = elements.into_iter().collect();
        let motion_in_use_elsewhere = attached_to.into_iter().any(|element_id| !elements.contains(&&element_id));

        if motion_in_use_elsewhere {
            // Create a new translation motion and attach/detach our elements (so elements outside of our set are not moved)
            let new_motion_id       = animation.motion().assign_motion_id();
            let detach_elements     = elements.iter().map(|element_id| AnimationEdit::Motion(motion_id, MotionEdit::Detach(**element_id)));
            let attach_elements     = elements.iter().map(|element_id| AnimationEdit::Motion(new_motion_id, MotionEdit::Attach(**element_id)));

            let create_new_motion   = vec![
                MotionEdit::Create,
                MotionEdit::SetType(MotionType::Translate),
                MotionEdit::SetOrigin(origin.0, origin.1),
                MotionEdit::SetPath(updated_curve)
            ];

            // Paste together to create the overall editing action (creating a new motion)
            create_new_motion.into_iter()
                .map(|motion_edit| AnimationEdit::Motion(new_motion_id, motion_edit))
                .chain(detach_elements)
                .chain(attach_elements)
                .collect()
        } else {
            // Move the existing curve
            vec![
                AnimationEdit::Motion(motion_id, MotionEdit::SetPath(updated_curve))
            ]
        }
    } else {
        // No edits if the motion doesn't exist or is not a translation
        vec![]
    }
}

///
/// Generates a move elements edit for a particular animation
/// 
fn move_elements_edit<Anim: Animation>(animation: &Anim, elements: &Vec<ElementId>, when: &Duration, from: &(f32, f32), to: &(f32, f32)) -> Vec<AnimationEdit> {
    // An element can either have an existing translation or have no translation attached to it yet
    let mut existing_translations   = HashMap::new();
    let mut static_elements         = HashSet::new();
    
    // Find the elements with existing translations and those without any
    for element_id in elements.iter() {
        // Get the motions for this element
        // TODO: it's a waste of time to re-fetch a motion that's already been fetched because it's attached to another element
        let element_motions         = animation.motion().get_motions_for_element(*element_id)
            .into_iter()
            .filter_map(|motion_id| animation.motion().get_motion(motion_id).map(|motion| (motion_id, motion)));
        
        // Filter to the translation motions
        let translation_motions     = element_motions
            .filter(|&(ref _id, ref motion)| motion.motion_type() == MotionType::Translate);
        let translation_motions: Vec<_> = translation_motions.collect();

        if translation_motions.len() == 0 {
            // No translation motions: this is a static element
            static_elements.insert(*element_id);
        } else {
            // Some translation motions already exist: these will need to be updated
            for (motion_id, _motion) in translation_motions {
                existing_translations.entry(motion_id)
                    .or_insert_with(|| vec![])
                    .push(*element_id);
            }
        }
    }

    // The existing translations need to be updated (and forked with new IDs if they're attached to elements not being edited)
    let move_existing = existing_translations.into_iter()
        .flat_map(|(motion_id, elements)| dynamic_move_edit(animation, motion_id, &elements, when, from, to));

    // The static elements need to have a translation attached (we attach the same translation to all)
    let move_static = static_move_edit(animation, &static_elements, when, from, to);

    // Combine the edits into the final reuslt
    move_existing
        .chain(move_static.into_iter())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::*;

    use futures::*;
    use std::ops::{Range, Deref};

    #[test]
    fn move_static_element() {
        // Test animation that has no attached motions
        struct TestAnimation;

        impl Animation for TestAnimation {
            fn size(&self) -> (f64, f64) { unimplemented!() }
            fn duration(&self) -> Duration { unimplemented!() }
            fn frame_length(&self) -> Duration { unimplemented!() }
            fn get_layer_ids(&self) -> Vec<u64> { unimplemented!() }
            fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<'a+Deref<Target='a+Layer>>> { unimplemented!() }
            fn get_num_edits(&self) -> usize { unimplemented!() }
            fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> { unimplemented!() }
            fn motion<'a>(&'a self) -> &'a AnimationMotion { self }
        }

        impl AnimationMotion for TestAnimation {
            fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>> { unimplemented!() }

            fn assign_motion_id(&self) -> ElementId {
                ElementId::Assigned(42)
            }

            fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
                vec![]
            }

            fn get_elements_for_motion(&self, _motion_id: ElementId) -> Vec<ElementId> {
                vec![]
            }

            fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
                None
            }
        }

        // Try to generate the edits for moving an element with this test animation
        let animation   = TestAnimation;
        let static_move = MotionEditAction::MoveElements(vec![ElementId::Assigned(1), ElementId::Assigned(2)], Duration::from_millis(442), (100.0, 200.0), (300.0, 400.0))
            .to_animation_edits(&animation);

        let target_point = TimePoint::new(300.0, 400.0, Duration::from_millis(442));

        assert!(static_move[0] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Create));
        assert!(static_move[1] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::SetType(MotionType::Translate)));
        assert!(static_move[2] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::SetOrigin(100.0, 200.0)));
        assert!(static_move[3] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::SetPath(TimeCurve::new(target_point, target_point))));

        // Attaching can be in either order
        if (static_move[4] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Attach(ElementId::Assigned(1)))) {
            assert!(static_move[5] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Attach(ElementId::Assigned(2))));
        } else {
            assert!(static_move[5] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Attach(ElementId::Assigned(1))));
            assert!(static_move[4] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Attach(ElementId::Assigned(2))));
        }

        assert!(static_move.len() == 6);
    }

    #[test]
    fn move_moving_element() {
        // Test animation that has no attached motions
        struct TestAnimation;

        impl Animation for TestAnimation {
            fn size(&self) -> (f64, f64) { unimplemented!() }
            fn duration(&self) -> Duration { unimplemented!() }
            fn frame_length(&self) -> Duration { unimplemented!() }
            fn get_layer_ids(&self) -> Vec<u64> { unimplemented!() }
            fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<'a+Deref<Target='a+Layer>>> { unimplemented!() }
            fn get_num_edits(&self) -> usize { unimplemented!() }
            fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> { unimplemented!() }
            fn motion<'a>(&'a self) -> &'a AnimationMotion { self }
        }

        impl AnimationMotion for TestAnimation {
            fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>> { unimplemented!() }

            fn assign_motion_id(&self) -> ElementId {
                ElementId::Assigned(43)
            }

            fn get_motions_for_element(&self, _element_id: ElementId) -> Vec<ElementId> {
                vec![ElementId::Assigned(42)]
            }

            fn get_elements_for_motion(&self, _motion_id: ElementId) -> Vec<ElementId> {
                vec![ElementId::Assigned(1)]
            }

            fn get_motion(&self, _motion_id: ElementId) -> Option<Motion> {
                Some(Motion::Translate(TranslateMotion::move_to(Duration::from_millis(442), (10.0, 30.0), (30.0, 40.0))))
            }
        }

        // Try to generate the edits for moving an element with this test animation
        let animation       = TestAnimation;
        let dynamic_move    = MotionEditAction::MoveElements(vec![ElementId::Assigned(1), ElementId::Assigned(2)], Duration::from_millis(442), (100.0, 200.0), (300.0, 400.0))
            .to_animation_edits(&animation);

        let target_point = TimePoint::new(220.0, 210.0, Duration::from_millis(442));

        assert!(dynamic_move[0] == AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::SetPath(TimeCurve::new(target_point, target_point))));

        assert!(dynamic_move.len() == 1);
    }
}
