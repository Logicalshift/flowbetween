use super::properties::*;

use canvas::*;

use std::time::Duration;
use std::any::*;

///
/// Represents an element in a vector layer
///
pub trait VectorElement : Send+Any {
    ///
    /// When this element should be drawn on the layer (relative to the start of the key frame)
    /// 
    fn appearance_time(&self) -> Duration;

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties);

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, _properties: &mut VectorProperties) { }
}
