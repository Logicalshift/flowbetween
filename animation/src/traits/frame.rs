use super::edit::*;
use super::vector::*;
use super::brush_definition::*;
use super::brush_properties::*;
use super::brush_drawing_style::*;

use canvas::*;

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
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    /// 
    fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>>;

    ///
    /// Retrieves a copy of the element with the specifed ID from this frame, if it exists
    /// 
    fn element_with_id<'a>(&'a self, id: ElementId) -> Option<Vector>;

    ///
    /// The brush that is active after all the elements are drawn in this frame
    /// 
    /// (If new elements are added to the layer at the time index of this frame,
    /// this is the brush that will be used)
    /// 
    fn active_brush(&self) -> Option<(BrushDefinition, BrushDrawingStyle)>;

    ///
    /// The brush properties that are active after all the elements are drawn
    /// in this frame.
    /// 
    /// (If new elements are added to the layer at the time index of this frame,
    /// these are the properties that will be used)
    /// 
    fn active_brush_properties(&self) -> Option<BrushProperties>;
}
