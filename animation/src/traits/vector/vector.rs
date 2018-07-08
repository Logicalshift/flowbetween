use super::element::*;
use super::brush_element::*;
use super::transformed_vector::*;
use super::brush_properties_element::*;
use super::brush_definition_element::*;
use super::super::edit::ElementId;

use std::ops::Deref;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    /// Vector that has been transformed from a source vector (eg, by applying a motion)
    Transformed(TransformedVector),

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
    type Target = dyn VectorElement;

    #[inline]
    fn deref(&self) -> &dyn VectorElement {
        use Vector::*;

        match self {
            &Transformed(ref transform) => transform,

            &BrushDefinition(ref defn)  => defn,
            &BrushProperties(ref props) => props,
            &BrushStroke(ref elem)      => elem
        }
    }
}
