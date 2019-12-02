use super::element::*;
use super::path_element::*;
use super::brush_element::*;
use super::group_element::*;
use super::motion_element::*;
use super::transformed_vector::*;
use super::brush_properties_element::*;
use super::brush_definition_element::*;
use super::super::edit::ElementId;

use std::ops::Deref;

///
/// Possible types of vector element
///
#[derive(Clone, Debug)]
pub enum Vector {
    /// Vector that has been transformed from a source vector (eg, by applying a motion)
    Transformed(TransformedVector),

    /// Sets the brush to use for future brush strokes
    BrushDefinition(BrushDefinitionElement),

    /// Brush properties for future brush strokes
    BrushProperties(BrushPropertiesElement),

    /// Brush stroke vector
    BrushStroke(BrushElement),

    /// Path vector
    Path(PathElement),

    /// Element describing a motion
    Motion(MotionElement),

    /// Element describing a group (with optional cache and path combining operation)
    Group(GroupElement)
}

impl Vector {
    ///
    /// Creates a new vector from an element
    ///
    #[inline]
    pub fn new<IntoVec: Into<Vector>>(from: IntoVec) -> Vector {
        from.into()
    }

    ///
    /// The ID for this vector
    ///
    #[inline]
    pub fn id(&self) -> ElementId {
        self.deref().id()
    }

    ///
    /// If this element was transformed from an original element, returns that original element
    ///
    pub fn original_without_transformations(&self) -> Vector {
        use self::Vector::*;

        match self {
            Transformed(transformed)    => (*transformed.without_transformations()).clone(),
            not_transformed             => not_transformed.clone()
        }
    }
}

impl Deref for Vector {
    type Target = dyn VectorElement;

    #[inline]
    fn deref(&self) -> &dyn VectorElement {
        use Vector::*;

        match self {
            Transformed(transform)          => transform,

            BrushDefinition(defn)           => defn,
            BrushProperties(props)          => props,
            BrushStroke(elem)               => elem,

            Path(elem)                      => elem,
            Motion(elem)                    => elem,
            Group(elem)                     => elem
        }
    }
}
