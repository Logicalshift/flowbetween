use super::control::*;
use super::attributes::*;

use modifier::*;

///
/// Hints that can be applied to a control
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Hint {
    // Prefer fast drawing over correctness
    FastDrawing
}

impl Modifier<Control> for Hint {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::HintAttr(self))
    }
}
