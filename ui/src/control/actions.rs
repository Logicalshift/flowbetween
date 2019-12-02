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

    /// Tracks drag actions for this control
    Drag,

    /// This item has been focused for editing
    Focused,

    /// The value of an item is being edited and has a new intermediate value
    EditValue,

    /// The value of an item has been edited and should be updated
    SetValue,

    /// An edit (which may have sent one or more EditValue updates) has been cancelled
    CancelEdit,

    /// Divides a scrollable region into a grid, and generates an event whenever the region in the top-left corner changes
    VirtualScroll(f32, f32)
}

///
/// Indicates what type of drag action is occurring
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum DragAction {
    Start   = 0,
    Drag    = 1,
    Finish  = 2,
    Cancel  = 3
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

    /// Item drag action. Coordinates are relative to a fixed point during a drag action
    Drag(DragAction, (f32, f32), (f32, f32)),

    /// The new value for an item
    Value(PropertyValue),

    /// Indicates the top-left corner of a scrollable region (as a grid coordinate)
    /// and the size of the region (in grid entries)
    ///
    /// Ie, if you set a grid size of 512, 512, you might get '2,1' as the first pair
    /// to indicate that  the top-left corner is the grid square 1024, 512. A value
    /// of 3, 2 in the second would indicate that the client area of the scroll
    /// region is 1536x1024 (ie, you need to draw 3 512x512 squares horizontally
    /// and 2 vertically in order to cover everything the user can currently see)
    VirtualScroll((u32, u32), (u32, u32))
}
