use super::control::*;
use super::attributes::*;

use modifier::*;

///
/// Attributes for fonts
/// 
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Font {
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

impl Modifier<Control> for Font {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(self))
    }
}

impl<'a> Modifier<Control> for &'a Font {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(*self))
    }
}

impl Modifier<Control> for TextAlign {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(Font::Align(self)))
    }
}

impl<'a> Modifier<Control> for &'a TextAlign {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(Font::Align(*self)))
    }
}
