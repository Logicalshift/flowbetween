use super::edit::*;
use super::vector::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a single frame in a layer of an animation
///
pub trait Frame : Send+Sync {
    ///
    /// Time index of this frame relative to its keyframe
    ///
    fn time_index(&self) -> Duration;

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut dyn GraphicsPrimitives);

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>) -> Arc<VectorProperties>;

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    ///
    fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>>;

    ///
    /// Retrieves a copy of the element with the specifed ID from this frame, if it exists
    ///
    fn element_with_id(&self, id: ElementId) -> Option<Vector>;

    ///
    /// Retrieves the IDs and types of the elements attached to the element with a particular ID
    ///
    /// (Element data can be retrieved via element_with_id)
    ///
    fn attached_elements(&self, id: ElementId) -> Vec<(ElementId, VectorType)>;
}

impl Frame for Arc<dyn Frame> {
    ///
    /// Time index of this frame relative to its keyframe
    ///
    #[inline] fn time_index(&self) -> Duration { (**self).time_index() }

    ///
    /// Renders this frame to a particular graphics context
    ///
    #[inline] fn render_to(&self, gc: &mut dyn GraphicsPrimitives) { (**self).render_to(gc) }

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    #[inline] fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { (**self).apply_properties_for_element(element, properties) }

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    ///
    #[inline] fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>> { (**self).vector_elements() }

    ///
    /// Retrieves a copy of the element with the specifed ID from this frame, if it exists
    ///
    #[inline] fn element_with_id(&self, id: ElementId) -> Option<Vector> { (**self).element_with_id(id) }

    ///
    /// Retrieves the IDs and types of the elements attached to the element with a particular ID
    ///
    /// (Element data can be retrieved via element_with_id)
    ///
    fn attached_elements(&self, id: ElementId) -> Vec<(ElementId, VectorType)> { (**self).attached_elements(id) }
}
