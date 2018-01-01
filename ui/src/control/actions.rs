use super::paint::*;
use super::super::property::*;

///
/// Description of what should trigger an action
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ActionTrigger {
    /// User clicked this item (pressed down and released while over the same item)
    Click,

    /// Tracks all user clicks and drags over this item with a particular device
    /// In the event a control has multiple devices associated with it, we only track
    /// paint strokes from a single device (ie, you have to finish a paint stroke before
    /// you can begin a new one with a different input method)
    Paint(PaintDevice),

    /// The value of an item is being edited and has a new intermediate value
    EditValue,

    /// The value of an item has been edited and should be updated
    SetValue
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
    Value(PropertyValue)
}
