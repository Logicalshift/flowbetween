use super::super::property::*;

///
/// Represents a position coordinate
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Position {
    /// Point located at a specific value
    At(f32),

    /// Point located at a value specified by a property
    AtProperty(Property),

    /// Point at an offset from its counterpart (eg, width or height)
    Offset(f32),

    /// As a final point, stretches with the specified ratio to other stretch controls
    Stretch(f32),

    /// Point located at the start of the container (ie, left or top depending on if this is an x or y position)
    Start,

    /// Control located at the end of its container (ie, right or bottom depending on if this is an x or y position)
    End,

    /// Same as the last point in this axis (which is 0 initially)
    After
}
