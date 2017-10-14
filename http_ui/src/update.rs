use super::viewmodel::*;

use ui::control::*;

///
/// Describes a HTML node that should be changed
///
#[derive(Clone, Serialize, Deserialize)]
pub struct HtmlDiff {
    /// The address in the document of the node to be replaced
    address: Vec<u32>,

    /// The HTML that should replace this node
    new_html: String
}

impl HtmlDiff {
    ///
    /// Creates a new HTML diff
    ///
    pub fn new(address: Vec<u32>, new_html: String) -> HtmlDiff {
        HtmlDiff {
            address: address,
            new_html: String::from(new_html)
        }
    }
}

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
    /// Supplies a new user interface as HTML, alongside the corresponding UI control data
    ///
    NewUserInterfaceHtml(String, Control, Vec<ViewModelUpdate>),

    ///
    /// Specifies that the viewmodel should be updated
    ///
    UpdateViewModel(ViewModelUpdate),

    ///
    /// Specifies how the HTML should be updated
    ///
    UpdateHtml(Vec<HtmlDiff>),

    ///
    /// Replace the SVG element with the specified ID with the supplied SVG
    /// 
    /// Parameters are the ID and the replacement SVG data
    ///
    ReplaceSvg(String, String)
}
