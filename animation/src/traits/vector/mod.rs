use canvas::*;

mod properties;
mod path;
mod element;
mod brush_element;
mod brush_properties_element;
mod brush_definition_element;

pub use self::properties::*;
pub use self::path::*;
pub use self::element::*;
pub use self::brush_element::*;
pub use self::brush_properties_element::*;
pub use self::brush_definition_element::*;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    /// Empty vector
    Empty,

    /// Brush stroke vector
    Brush(BrushElement)
}

impl VectorElement for Vector {
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        match self {
            &Vector::Empty              => (),
            &Vector::Brush(ref elem)    => elem.render(gc, properties)
        }
    }

    fn update_properties(&self, properties: &mut VectorProperties) { 
        match self {
            &Vector::Empty              => (),
            &Vector::Brush(ref elem)    => elem.update_properties(properties)
        }
    }
}
