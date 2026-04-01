use super::*;
use super::super::property::*;

use ::modifier::*;

///
/// Attributes representing the state of a control
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum State {
    /// Whether or not this control is selected
    Selected(Property),

    /// Whether or not this control has a badge attached to it
    Badged(Property),

    /// Whether or not this control is enabled
    Enabled(Property),

    /// The value of this control (when it is not being edited)
    Value(Property),

    /// The range values that this control can be set to
    Range((Property, Property)),

    /// The priority for focusing this control (if it's created while no control is focused or a control with a lower priority is focused, then this control will be focused instead)
    FocusPriority(Property)
}

impl Modifier<Control> for State {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::StateAttr(self))
    }
}

impl<'a> Modifier<Control> for &'a State {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::StateAttr(self.clone()))
    }
}
