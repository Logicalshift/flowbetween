use super::element_id::*;
use super::super::motion::*;
use super::super::time_path::*;

use smallvec::*;

///
/// Represents an edit that creates a motion description on a layer
///
#[derive(Clone, PartialEq, Debug)]
pub enum MotionEdit {
    /// Creates a new motion with this element ID
    ///
    /// A new motion is created with a type of `None`, an origin at 0,0 and an empty time curve
    Create,

    /// Deletes the motion with this ID
    Delete,

    /// Sets the type of this motion
    SetType(MotionType),

    /// Changes the origin point for this motion
    SetOrigin(f32, f32),

    /// Sets the time curve for this motion
    SetPath(TimeCurve),
}

impl MotionEdit {
    ///
    /// Retrieves the element IDs used by this edit
    ///
    #[inline]
    pub fn used_element_ids(&self) -> SmallVec<[ElementId; 4]> {
        use MotionEdit::*;

        match self {
            Create          => smallvec![],
            Delete          => smallvec![],
            SetType(_)      => smallvec![],
            SetOrigin(_, _) => smallvec![],
            SetPath(_)      => smallvec![],
        }
    }
}
