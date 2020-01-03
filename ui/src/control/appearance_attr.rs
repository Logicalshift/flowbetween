use super::*;
use super::super::image;
use super::super::resource_manager::*;

use flo_canvas;
use ::modifier::*;

///
/// Attributes that describe the appearance of a control
///
#[derive(Clone, PartialEq, Debug)]
pub enum Appearance {
    /// Specifies the foreground colour of this control
    Foreground(flo_canvas::Color),

    /// Specifies the background colour of this control
    Background(flo_canvas::Color),

    /// Specifies the background image for this control
    Image(Resource<image::Image>)
}

impl Modifier<Control> for Appearance {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::AppearanceAttr(self))
    }
}

impl<'a> Modifier<Control> for &'a Appearance {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::AppearanceAttr(self.clone()))
    }
}

impl Modifier<Control> for Resource<image::Image> {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::AppearanceAttr(Appearance::Image(self)))
    }
}
