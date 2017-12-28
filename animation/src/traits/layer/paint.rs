use super::super::brush::*;
use super::super::brush_definition::*;

use ui::canvas::*;

use std::time::Duration;

///
/// Represents a layer that can be painted upon
/// 
pub trait PaintLayer : Send+Sync {
    ///
    /// Selects the brush that will be used for future brush strokes
    /// 
    fn select_brush(&mut self, brush: &BrushDefinition);

    ///
    /// Starts a new brush stroke on this layer
    /// 
    /// The start_time indicates when the brush stroke will appear along the timeline.
    /// It's valid for it not to fall on a key frame (in which case the brush stroke will
    /// be added to the existing frame after the specified time period has passed)
    ///
    fn start_brush_stroke(&mut self, start_time: Duration, initial_pos: BrushPoint);

    ///
    /// Continues a brush stroke on this layer
    /// 
    fn continue_brush_stroke(&mut self, point: BrushPoint);

    ///
    /// Finishes the current brush stroke
    /// 
    fn finish_brush_stroke(&mut self);

    ///
    /// Cancels the current brush stroke
    /// 
    fn cancel_brush_stroke(&mut self);

    ///
    /// Draws the in-progress brush stroke to the specified graphics context
    /// 
    fn draw_current_brush_stroke(&self, gc: &mut GraphicsPrimitives);

    ///
    /// Returns true if this item has an active brush stroke (not cancelled or finished)
    /// 
    fn has_pending_brush_stroke(&self) -> bool;
}
