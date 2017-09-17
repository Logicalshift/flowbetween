///
/// Represents details of an event from the browser side
///
#[derive(Clone, Serialize, Deserialize)]
pub enum Event {
    ///
    /// The user clicked on a named SVG element (in a given spot)
    ///
    ClickSvg(String, f32, f32)
}