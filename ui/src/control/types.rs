///
/// Possible types of control
///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ControlType {
    /// A control that does nothing
    Empty,

    /// Control that contains other controls
    Container,

    /// Control that contains other controls (and crops them to the bounds of this control)
    CroppingContainer,

    /// Control that contains other controls and some scroll bars
    ScrollingContainer,

    /// Control that 'pops up' from its parent, usually a temporary
    /// dialog box of some description
    Popup,

    /// Clickable button
    Button,

    /// Label used to display some text
    Label,

    /// Allows arbitrary rendering using a canvas resource
    Canvas,

    /// Allows picking a value by dragging left or right
    Slider,

    /// A circular slider that represents its value by how much it is rotated
    Rotor,

    /// A single-line text editor
    TextBox,

    /// A checkbox that can be turned on or off, with optional label
    CheckBox
}
