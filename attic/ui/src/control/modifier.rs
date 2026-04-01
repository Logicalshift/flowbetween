use super::control::*;
use super::attributes::*;

use ::modifier::*;

pub trait ControlModifier {
    fn modify(self, control: &mut Control);
}

impl Modifier<Control> for ControlAttribute {
    fn modify(self, control: &mut Control) {
        control.add_attribute(self);
    }
}

impl<A: Modifier<Control>> ControlModifier for Option<A> {
    fn modify(self, control: &mut Control) {
        if let Some(modifier) = self {
            modifier.modify(control);
        }
    }
}

impl<A: Modifier<Control>> ControlModifier for A {
    fn modify(self, control: &mut Control) {
        self.modify(control)
    }
}
