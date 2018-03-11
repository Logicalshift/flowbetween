use super::properties::*;
use super::super::path::*;

use canvas::*;

use std::any::*;

///
/// Represents an element in a vector layer
///
pub trait VectorElement : Send+Any {
    ///
    /// The ID of this vector element
    /// 
    fn id(&self) -> u64;

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>>;

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties);

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, _properties: &mut VectorProperties) { }
}
