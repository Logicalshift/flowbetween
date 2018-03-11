use super::element_id::*;

use super::super::raw_point::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;

use std::sync::*;

///
/// Represents an edit involving painting
///
#[derive(Clone, PartialEq, Debug)]
pub enum PaintEdit {
    /// Selects the brush with the specified definition for painting
    SelectBrush(ElementId, BrushDefinition, BrushDrawingStyle),

    /// Sets the properties for brush strokes
    BrushProperties(ElementId, BrushProperties),

    /// Draws a brush stroke using the current brush and the specified set of input points
    BrushStroke(ElementId, Arc<Vec<RawPoint>>)
}
