use super::path_element::*;
use super::brush_element::*;
use super::shape_element::*;
use super::group_element::*;
use super::error_element::*;
use super::motion_element::*;
use super::vector_element::*;
use super::transformation::*;
use super::animation_element::*;
use super::transformed_vector::*;
use super::brush_properties_element::*;
use super::brush_definition_element::*;
use super::super::path::*;
use super::super::edit::ElementId;

use smallvec::*;

use std::ops::{Deref, DerefMut};
use std::sync::*;
use std::iter;

///
/// Possible types of vector element
///
#[derive(Clone, PartialEq, Debug)]
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

    /// Shape
    Shape(ShapeElement),

    /// Element describing a motion
    /// TODO: these are obsolete and replaced by animation regions
    Motion(MotionElement),

    /// Element describing a group (with optional cache and path combining operation)
    Group(GroupElement),

    /// Attached to an element to indicate a transformation that should be applied to it when rendering
    Transformation((ElementId, SmallVec<[Transformation; 2]>)),

    /// An element representing an animation region for the keyframe
    AnimationRegion(AnimationElement),

    /// Element exists but could not be loaded from the file
    Error
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
    /// Creates a new vector with a particular ID
    ///
    #[inline]
    pub fn with_id(&self, new_id: ElementId) -> Vector {
        let mut new_vector = self.clone();
        new_vector.set_id(new_id);
        new_vector
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

    ///
    /// If this is a brush definition element, returns that element (otherwise drops the vector)
    ///
    pub fn extract_brush_definition(self) -> Option<BrushDefinitionElement> {
        match self {
            Vector::BrushDefinition(elem)   => Some(elem),
            _                               => None
        }
    }

    ///
    /// If this is a brush properties element, returns that element (otherwise drops the vector)
    ///
    pub fn extract_brush_properties(self) -> Option<BrushPropertiesElement> {
        match self {
            Vector::BrushProperties(elem)   => Some(elem),
            _                               => None
        }
    }

    ///
    /// Creates an updated vector element with updated path components
    ///
    pub fn with_path_components(&self, path_components: impl IntoIterator<Item=PathComponent>) -> Vector {
        match self {
            Vector::Path(path_element) => {
                // Create a clone of the path element with the new properties
                let new_path            = Path::from_elements(path_components);
                let new_path_element    = PathElement::new(path_element.id(), new_path);

                Vector::new(new_path_element)
            },

            // Element is unchanged if it's not a path
            _ => self.clone()
        }
    }

    ///
    /// Retrieves the path components that make up this vector element
    ///
    pub fn path_components(&self) -> Arc<Vec<PathComponent>> {
        match self {
            Vector::Path(path_element)  => path_element.path().elements.clone(),
            _                           => Arc::new(vec![]),
        }
    }

    ///
    /// Returns an iterator of any sub-elements this vector has
    ///
    pub fn sub_elements<'a>(&'a self) -> impl 'a+Iterator<Item=&Vector> {
        let result: Box<dyn 'a+Iterator<Item=&'a Vector>> = match self {
            Vector::Group(group_elem) => {
                Box::new(group_elem.elements())
            }

            _ => Box::new(iter::empty())
        };

        result
    }

    ///
    /// The IDs of the sub-elements of this element
    ///
    pub fn sub_element_ids(&self) -> Vec<ElementId> {
        let sub_elements    = self.sub_elements();
        let sub_element_ids = sub_elements.map(|elem| elem.id()).collect();

        sub_element_ids
    }

    ///
    /// If this vector contains subelements (eg, is a group), this is the value of the 'topmost' element that it contains
    ///
    pub fn topmost_sub_element(&self) -> Option<ElementId> {
        match self {
            Vector::Group(group_elem)   => {
                group_elem.elements().last()
                    .map(|elem| elem.id())
            }
            _                           => None
        }
    }
}

impl DerefMut for Vector {
    #[inline]
    fn deref_mut(&mut self) -> &mut dyn VectorElement {
        use Vector::*;

        match self {
            Transformed(transform)          => transform,

            BrushDefinition(defn)           => defn,
            BrushProperties(props)          => props,
            BrushStroke(elem)               => elem,

            Path(elem)                      => elem,
            Shape(elem)                     => elem,
            Motion(elem)                    => elem,
            Group(elem)                     => elem,
            Transformation(elem)            => elem,
            AnimationRegion(elem)           => elem,
            Error                           => panic!("Cannot edit an error element")
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
            Shape(elem)                     => elem,
            Motion(elem)                    => elem,
            Group(elem)                     => elem,
            AnimationRegion(elem)           => elem,
            Transformation(transform)       => transform,
            Error                           => &*ERROR_ELEMENT
        }
    }
}
