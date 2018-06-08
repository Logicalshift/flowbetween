use super::vector::*;
use super::properties::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;

use canvas::*;

use std::time::Duration;
use std::sync::*;
use std::any::*;

///
/// Represents a vector element in a frame
///
pub trait VectorElement : Send+Any {
    ///
    /// The ID of this element
    /// 
    fn id(&self) -> ElementId;

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>>;

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties);

    ///
    /// Returns the properties to use for future elements
    /// 
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { properties }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector;
}
