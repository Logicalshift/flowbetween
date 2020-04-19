use super::element_id::*;

use super::super::path::*;
use super::super::raw_point::*;
use super::super::fill_option::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;

use std::sync::*;

///
/// Represents an edit to a path
///
#[derive(Clone, PartialEq, Debug)]
pub enum PathEdit {
    /// Creates a new path consisting of the specified path components
    CreatePath(ElementId, Arc<Vec<PathComponent>>),

    /// Selects the brush with the specified definition for painting paths
    SelectBrush(ElementId, BrushDefinition, BrushDrawingStyle),

    /// Sets the properties for brush strokes
    BrushProperties(ElementId, BrushProperties),
}

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
    BrushStroke(ElementId, Arc<Vec<RawPoint>>),

    /// Creates a path by flood-filling at the specified point on the current layer. The current brush/properties are used to generate
    /// the fill path, and some other options can be set in the fill options.
    Fill(ElementId, RawPoint, Vec<FillOption>)
}

impl PaintEdit {
    ///
    /// The element ID for this edit
    ///
    pub fn id(&self) -> ElementId {
        use self::PaintEdit::*;

        match self {
            SelectBrush(id, _, _)   => *id,
            BrushProperties(id, _)  => *id,
            BrushStroke(id, _)      => *id,
            Fill(id, _, _)          => *id
        }
    }

    ///
    /// If this edit contains an unassigned element ID, calls the specified function to supply a new
    /// element ID. If the edit already has an ID, leaves it unchanged.
    ///
    pub fn assign_element_id<AssignFn: FnOnce() -> i64>(self, assign_element_id: AssignFn) -> PaintEdit {
        use self::PaintEdit::*;
        use self::ElementId::*;

        match self {
            SelectBrush(Unassigned, brush_def, brush_style) => SelectBrush(Assigned(assign_element_id()), brush_def, brush_style),
            BrushProperties(Unassigned, brush_props)        => BrushProperties(Assigned(assign_element_id()), brush_props),
            BrushStroke(Unassigned, points)                 => BrushStroke(Assigned(assign_element_id()), points),

            assigned => assigned
        }
    }
}
