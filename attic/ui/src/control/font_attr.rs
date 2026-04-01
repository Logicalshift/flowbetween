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
    Align(TextAlign),

    /// Font weight
    Weight(FontWeight)
}

///
/// Represents the weight of a font
///
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FontWeight {
    Light       = 100,
    Normal      = 400,
    Bold        = 700,
    ExtraBold   = 900
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

impl Modifier<Control> for FontWeight {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(Font::Weight(self)))
    }
}

impl<'a> Modifier<Control> for &'a FontWeight {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::FontAttr(Font::Weight(*self)))
    }
}
