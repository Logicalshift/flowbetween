
///
/// Represents an instruction to perform an update in the web interface
///
#[derive(Clone, Serialize, Deserialize)]
pub enum Update {
    ///
    /// Replace the SVG element with the specified ID with the supplied SVG
    /// 
    /// Parameters are the ID and the replacement SVG data
    ///
    ReplaceSvg(String, String)
}
