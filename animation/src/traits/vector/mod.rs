use super::edit::ElementId;

use canvas::*;

mod properties;
mod element;
mod brush_element;
mod brush_properties_element;
mod brush_definition_element;

pub use self::properties::*;
pub use self::element::*;
pub use self::brush_element::*;
pub use self::brush_properties_element::*;
pub use self::brush_definition_element::*;

use std::ops::Deref;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    /// Sets the brush properties for future brush strokes
    BrushDefinition(BrushDefinitionElement),

    /// Brush properties for future brush strokes
    BrushProperties(BrushPropertiesElement),

    /// Brush stroke vector
    BrushStroke(BrushElement)
}

impl Vector {
    ///
    /// Creates a new vector from an element
    /// 
    #[inline]
    pub fn new<IntoVec: Into<Vector>>(from: IntoVec) -> Vector {
        from.into()
    }

    #[inline]
    pub fn id(&self) -> ElementId {
        self.deref().id()
    }
}

impl Deref for Vector {
    type Target = VectorElement;

    #[inline]
    fn deref(&self) -> &VectorElement {
        use Vector::*;

        match self {
            &BrushDefinition(ref defn)  => defn,
            &BrushProperties(ref props) => props,
            &BrushStroke(ref elem)      => elem
        }
    }
}
