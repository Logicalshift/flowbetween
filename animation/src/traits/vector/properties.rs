use super::super::brush::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use std::sync::*;

///
/// Represents the active properties for a vector layer
/// 
/// Elements can update the properties, which persist to the next element.
/// This saves some space in that properties don't need to be encoded with
/// each element.
/// 
#[derive(Clone)]
pub struct VectorProperties {
    /// The active brush
    pub brush: Arc<Brush>,

    /// The properties set for the active brush
    pub brush_properties: BrushProperties
}

impl VectorProperties {
    ///
    /// Creates the default brush properties
    /// 
    pub fn default() -> VectorProperties {
        VectorProperties {
            brush:              Arc::new(InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw)),
            brush_properties:   BrushProperties::new()
        }
    }
}