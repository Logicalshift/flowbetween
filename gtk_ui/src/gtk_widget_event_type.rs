///
/// Types of widget event that can be registered
/// 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GtkWidgetEventType {
    /// User pressed and released the mouse over a particular widget (or any of its children, if they do not generate their own event for this situation)
    Click,

    /// User is in the process of editing a value
    EditValue,

    /// User has picked a final value
    SetValue
}
