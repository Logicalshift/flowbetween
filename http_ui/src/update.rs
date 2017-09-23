
///
/// Represents an instruction to perform an update in the web interface
///
/// Events and other requests to the HTTP interface can return lists
/// of updates that should be performed in response.
///
#[derive(Clone, Serialize, Deserialize)]
pub enum Update {
    ///
    /// There is no session ID or the session is unknown
    ///
    MissingSession,

    ///
    /// A new session has been created, and this is its ID
    ///
    NewSession(String),

    ///
    /// Supplies a new user interface as HTML
    ///
    NewUserInterfaceHtml(String),

    ///
    /// Replace the SVG element with the specified ID with the supplied SVG
    /// 
    /// Parameters are the ID and the replacement SVG data
    ///
    ReplaceSvg(String, String)
}
