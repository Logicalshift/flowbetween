use super::element_id::*;
use super::super::motion::*;
use super::super::time_path::*;

///
/// Represents an edit that creates a motion description on a layer
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum MotionEdit {
    /// Creates a new motion with this ID
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

    /// Attaches this motion to the specified element ID
    Attach(ElementId),

    /// Detaches this motion from the specified element ID
    Detach(ElementId)
}
