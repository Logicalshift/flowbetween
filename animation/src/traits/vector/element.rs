use super::properties::*;

use canvas::*;

use std::any::*;

///
/// Represents an element in a vector layer
///
pub trait VectorElement : Send+Any {
    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties);

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, _properties: &mut VectorProperties) { }
}
