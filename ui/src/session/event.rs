use super::super::control::*;

///
/// Possible events that can be sent to the UI
/// 
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum UiEvent {
    /// Performs the specified action
    Action(Vec<String>, String, ActionParameter),

    /// Sends a tick to all the controllers
    Tick
}
