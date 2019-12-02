use super::canvas_update::*;

use ui::*;

use serde_json;

///
/// Describes a HTML node that should be changed
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HtmlDiff {
    /// The address in the document of the node to be replaced
    pub address: Vec<u32>,

    /// The UI tree corresponding to the new HTML
    pub ui_tree: serde_json::Value,

    /// The HTML that should replace this node
    pub new_html: String
}

impl HtmlDiff {
    ///
    /// Creates a new HTML diff
    ///
    pub fn new(address: Vec<u32>, ui_tree: &Control, new_html: String) -> HtmlDiff {
        HtmlDiff {
            address:    address,
            ui_tree:    ui_tree.to_json(),
            new_html:   String::from(new_html)
        }
    }
}

///
/// Represents an instruction to perform an update in the web interface
///
/// Events and other requests to the HTTP interface can return lists
/// of updates that should be performed in response.
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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
    /// The server supports Flo websockets on the specified port
    ///
    WebsocketPort(u32),

    ///
    /// The server supports Flo websockets on the same port as other requests
    ///
    WebsocketSamePort,

    ///
    /// Supplies a new user interface as HTML, alongside the corresponding UI control data
    /// and view model.
    ///
    NewUserInterfaceHtml(String, serde_json::Value, Vec<ViewModelUpdate>),

    ///
    /// Specifies that the viewmodel should be updated
    ///
    UpdateViewModel(Vec<ViewModelUpdate>),

    ///
    /// Specifies how the HTML should be updated
    ///
    UpdateHtml(Vec<HtmlDiff>),

    ///
    /// Specifies that a canvas should be updated
    ///
    UpdateCanvas(Vec<CanvasUpdate>)
}
