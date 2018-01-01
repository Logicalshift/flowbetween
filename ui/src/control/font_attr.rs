use super::control::*;
use super::attributes::*;

use modifier::*;

///
/// Attributes for fonts
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FontAttr {
    /// Font size in pixels
    Size(f32)
}

impl Modifier<Control> for FontAttr {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttribute(self))
    }
}

impl<'a> Modifier<Control> for &'a FontAttr {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttribute(*self))
    }
}
