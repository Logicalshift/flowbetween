use super::*;

use ::modifier::*;

///
/// Describes what happens to a control when the user moves the mouse pointer over it
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Hover {
    /// Tooltip to display when the mouse is left over this item (or one of its child items)
    Tooltip(String)
}

impl Modifier<Control> for Hover {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::HoverAttr(self))
    }
}

impl<'a> Modifier<Control> for &'a Hover {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::HoverAttr(self.clone()))
    }
}
