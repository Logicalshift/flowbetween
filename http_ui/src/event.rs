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
    UiRefresh
}