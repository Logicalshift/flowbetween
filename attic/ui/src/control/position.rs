use super::super::property::*;

///
/// Represents a position coordinate
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Position {
    /// Point located at a specific value
    At(f32),

    /// Point located at a value specified by a property, with an offset
    /// When this item is moved, only its bounding box is changed: no further
    /// layout is performed. Other items are laid out as if this had the offset
    /// value (this is what makes this a 'floating' item: it can float away
    /// from its initial position without affecting other items)
    ///
    /// This is useful for items that move but don't otherwise change shape:
    /// as no layout needs to be performed beyond moving the item these are always
    /// fast to change.
    Floating(Property, f32),

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
