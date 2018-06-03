use super::edit_action::*;
use super::super::edit::*;

use std::time::Duration;

///
/// Edit actions that cause objects to move
/// 
pub enum MotionEditAction {
    /// Moves a set of elements via a drag from a particular spot to another spot
    MoveElements(Vec<ElementId>, Duration, (f32, f32), (f32, f32))
}
