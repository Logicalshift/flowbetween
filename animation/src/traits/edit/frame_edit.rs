use super::super::brush::*;
use super::super::brush_properties::*;

///
/// Represents an edit involving painting
///
pub enum PaintEdit {
    /// Selects the brush with the specified ID for painting
    SelectBrush(u32),

    /// Sets the properties for brush strokes
    BrushProperties(BrushProperties),

    /// Draws a brush stroke using the current brush and the specified set of input points
    BrushStroke(Vec<BrushPoint>)
}
