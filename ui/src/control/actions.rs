///
/// Description of what should trigger an action
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionTrigger {
    /// User clicked this item (pressed down and released while over the same item)
    Click,

    /// Tracks all user clicks and drags over this item
    Paint
}
