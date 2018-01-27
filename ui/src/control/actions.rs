use super::paint::*;
use super::super::property::*;

///
/// Description of what should trigger an action
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ActionTrigger {
    /// User clicked this item (pressed down and released while over the same item)
    Click,

    /// User began an interaction outside of this item (usually means that a popup should be dismissed)
    Dismiss,

    /// Tracks all user clicks and drags over this item with a particular device
    /// In the event a control has multiple devices associated with it, we only track
    /// paint strokes from a single device (ie, you have to finish a paint stroke before
    /// you can begin a new one with a different input method)
    Paint(PaintDevice),

    /// The value of an item is being edited and has a new intermediate value
    EditValue,

    /// The value of an item has been edited and should be updated
    SetValue,

    /// Divides a scrollable region into a grid, and generates an event whenever the region in the top-left corner changes
    VirtualScroll(f32, f32)
}

///
/// Data that can be sent alongside an action
///
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum ActionParameter {
    /// Action has no extra data
    None,

    /// Painting information
    Paint(PaintDevice, Vec<Painting>),

    /// The new value for an item
    Value(PropertyValue),

    /// Indicates the top-left corner of a scrollable region (as a grid coordinate) and the size of the region (in grid entries)
    VirtualScroll((u32, u32), (u32, u32))
}
