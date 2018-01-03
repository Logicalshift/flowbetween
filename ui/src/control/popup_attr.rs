use super::*;
use super::super::property::*;

use modifier::*;

///
/// The direction in which the popup should be shown
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PopupDirection {
    /// Popup appears directly over the top of the parent control
    OnTop,

    /// Popup appears to the left of the parent control
    Left,

    /// Popup appears to the right of the parent control
    Right,

    /// Popup appears above the parent control
    Above,

    /// Popup appears below the parent control
    Below,

    /// Popup appears centered in the window
    WindowCentered,

    /// Popup appears to the left of the window
    WindowLeft,

    /// Popup appears to the right of the window
    WindowRight,

    /// Popup appears at the top of the window
    WindowTop,

    /// Popup appears at the bottom of the window
    WindowBottom
}

///
/// Attributes associated with controlling a popup
/// 
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Popup {
    /// Whether or not this popup is open (popups are closed by default)
    IsOpen(Property),

    /// The direction in which this popup should appear relative to its parent control
    Direction(PopupDirection),

    /// The size in pixels of this popup
    Size(u32, u32)
}

impl Modifier<Control> for Popup {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::PopupAttr(self))
    }
}

impl Modifier<Control> for PopupDirection {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::PopupAttr(Popup::Direction(self)))
    }
}

impl<'a> Modifier<Control> for &'a Popup {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::PopupAttr(self.clone()))
    }
}
