use super::control::*;
use super::attributes::*;

///
/// Represents an object that can be used to modify a control
/// 
pub trait ControlModifier {
    /// Applies this modifier to a control
    fn modify(self, control: &mut Control);
}

impl ControlModifier for ControlAttribute {
    fn modify(self, control: &mut Control) {
        control.add_attribute(self);
    }
}

impl<A: ControlModifier, B: ControlModifier> ControlModifier for (A, B) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
    }
}

impl<A: ControlModifier, B: ControlModifier, C: ControlModifier> ControlModifier for (A, B, C) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
        self.2.modify(control);
    }
}

impl<A: ControlModifier, B: ControlModifier, C: ControlModifier, D: ControlModifier> ControlModifier for (A, B, C, D) {
    fn modify(self, control: &mut Control) {
        self.0.modify(control);
        self.1.modify(control);
        self.2.modify(control);
        self.3.modify(control);
    }
}
