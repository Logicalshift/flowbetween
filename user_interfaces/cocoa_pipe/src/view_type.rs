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
            ScrollingContainer      => ViewType::Empty,
            Popup                   => ViewType::Empty,
            Button                  => ViewType::Button,
            Label                   => ViewType::Empty,
            Canvas                  => ViewType::Empty,
            Slider                  => ViewType::Empty,
            Rotor                   => ViewType::Empty,
            TextBox                 => ViewType::Empty,
            CheckBox                => ViewType::Empty
        }
    }
}
