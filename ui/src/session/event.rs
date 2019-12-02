use super::super::control::*;

///
/// Possible events that can be sent to the UI
///
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum UiEvent {
    /// Wait for the next resume before sending any further updates. This is used when we don't want intermediate states to be displayed to the UI during event processing
    SuspendUpdates,

    /// Resume a 'suspend' event
    ResumeUpdates,

    /// Performs the specified action: parameters are the controller path, the event name and the action parameter
    Action(Vec<String>, String, ActionParameter),

    /// Sends a tick to all the controllers. If updates are suspended, ticks are only sent when they resume,
    /// and only one tick is sent regardless of how many were requested during the suspension.
    Tick
}
