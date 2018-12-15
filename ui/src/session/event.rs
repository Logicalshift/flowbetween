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

    /// Performs the specified action
    Action(Vec<String>, String, ActionParameter),

    /// Sends a tick to all the controllers
    Tick
}
