use super::control::*;
use super::attributes::*;

use modifier::*;

///
/// Attributes for fonts
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FontAttr {
    /// Font size in pixels
    Size(f32),

    /// Horizontal alignment
    Align(TextAlign)
}

///
/// Represents the horizontal alignment for text
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right
}

// Modifiers for convenience

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

impl Modifier<Control> for TextAlign {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttribute(FontAttr::Align(self)))
    }
}

impl<'a> Modifier<Control> for &'a TextAlign {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttribute(FontAttr::Align(*self)))
    }
}
