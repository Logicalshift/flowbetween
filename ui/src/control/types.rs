///
/// Possible types of control
///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ControlType {
    /// A control that does nothing
    Empty,

    /// Control that contains other controls
    Container,

    /// Clickable button
    Button,

    /// Label used to display some text
    Label,

    /// Allows arbitrary rendering using a canvas resource
    Canvas,

    /// Allows picking a value by dragging left or right
    Slider
}
