use super::control::*;
use super::attributes::*;

use ::modifier::*;

///
/// Hints that can be applied to a control
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Hint {
    /// Prefer fast drawing over correctness
    FastDrawing,

    /// Set how pointer events are handled by this control
    PointerBehaviour(PointerBehaviour),

    /// Provides a class for this control (modifying its behaviour or appearance)
    Class(String)
}

///
/// Possible ways the pointer can be handled by this control
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PointerBehaviour {
    /// Clicks on this control block clicks to the control underneath (the default behaviour)
    BlockClicks,

    /// Clicks on this control go through to the control underneath
    ClickThrough
}

impl Modifier<Control> for Hint {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::HintAttr(self))
    }
}
