use flo_ui::*;

///
/// The supported types of Cocoa view
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewType {
    /// An empty view
    Empty,

    /// A view containing a button
    Button,

    /// A button that can contain other views (Cocoa has no native control to do this, so we draw this ourselves)
    ContainerButton,

    /// A view containing a slider
    Slider,

    /// A view that acts like a slider but operates by rotating its content
    Rotor,

    /// A view consisting of a text box
    TextBox,

    /// A view that lets you check or uncheck a value
    CheckBox,

    /// A view that can be scrolled
    Scrolling,

    /// A view that displays a popup window
    Popup
}

impl From<&Control> for ViewType {
    fn from(control: &Control) -> ViewType {
        ViewType::from(control.control_type())
    }
}

impl From<ControlType> for ViewType {
    fn from(control_type: ControlType) -> ViewType {
        use self::ControlType::*;

        match control_type {
            Empty                   => ViewType::Empty,
            Container               => ViewType::Empty,
            CroppingContainer       => ViewType::Empty,
            ScrollingContainer      => ViewType::Scrolling,
            Popup                   => ViewType::Popup,
            Button                  => ViewType::Button,
            Label                   => ViewType::Empty,
            Canvas                  => ViewType::Empty,
            Slider                  => ViewType::Slider,
            Rotor                   => ViewType::Rotor,
            TextBox                 => ViewType::TextBox,
            CheckBox                => ViewType::CheckBox
        }
    }
}
