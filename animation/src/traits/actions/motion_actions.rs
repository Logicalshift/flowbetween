use super::edit_action::*;
use super::super::edit::*;
use super::super::animation::*;

use std::time::Duration;
use std::collections::HashSet;

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
    MoveElements(Vec<ElementId>, Duration, (f32, f32), (f32, f32))
}

impl EditAction for MotionEditAction {
    ///
    /// Converts this edit action into a set of animation edits for a particular animation
    /// 
    fn to_animation_edits<Anim: Animation>(&self, animation: &Anim) -> Vec<AnimationEdit> {
        use self::MotionEditAction::*;

        match self {
            MoveElements(elements, when, from, to)  => move_elements(animation, elements, when, from, to)
        }
    }
}

///
/// Generates a move elements edit for a particular animation
/// 
fn move_elements<Anim: Animation>(animation: &Anim, elements: &Vec<ElementId>, when: &Duration, from: &(f32, f32), to: &(f32, f32)) -> Vec<AnimationEdit> {
    unimplemented!()
}
