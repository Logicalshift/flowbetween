use super::path::*;

use ui::canvas::*;

use std::time::Duration;
use std::any::*;

///
/// Represents an element in a vector layer
///
pub trait VectorElement : Any {
    ///
    /// When this element should be drawn on the layer (relative to the start of the key frame)
    /// 
    fn appearance_time(&self) -> Duration;

    ///
    /// Retrieves the path associated with this element
    /// 
    fn path(&self) -> Path;

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsContext);
}
