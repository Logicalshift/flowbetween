use ui::ActionParameter;

///
/// Represents details of an event from the browser side
///
#[derive(Clone, Serialize, Deserialize, Debug)]
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
    Action(Vec<String>, String, ActionParameter),

    ///
    /// Wait for the next resume before sending any further updates. This is used when we don't want intermediate states to be displayed to the UI during event processing
    ///
    SuspendUpdates,

    ///
    /// Resume a 'suspend' event
    ///
    ResumeUpdates,

    ///
    /// Sends a tick event to the controllers
    ///
    Tick
}
