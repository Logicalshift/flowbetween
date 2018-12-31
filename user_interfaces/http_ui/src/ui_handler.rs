use super::event::*;
use super::update::*;

///
/// Structure of a request sent to the UI handler
///
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UiHandlerRequest {
    /// The session ID, if there is one
    pub session_id: Option<String>,

    /// The events that the UI wishes to report with this request
    pub events: Vec<Event>
}

///
/// Structure of a UI handler response
///
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UiHandlerResponse {
    /// Updates generated for this request
    pub updates: Vec<Update>
}
