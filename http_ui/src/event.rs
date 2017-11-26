use ui::ActionParameter;

///
/// Represents details of an event from the browser side
///
#[derive(Clone, Serialize, Deserialize)]
pub enum Event {
    ///
    /// Request a new session
    ///
    NewSession,

    ///
    /// Request a refresh of the UI
    ///
    UiRefresh,

    ///
    /// Sends an action to the controller found along a certain path
    ///
    Action(Vec<String>, String, ActionParameter)
}