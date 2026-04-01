use flo_canvas::*;

///
/// Standard properties for a brush stroke
///
/// These are the properties that are independent of the brush type.
/// Properties that define a brush can be found in brush_definition.
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BrushProperties {
    /// The size of the brush stroke
    pub size: f32,

    /// The opacity of the brush stroke
    pub opacity: f32,

    /// The colour of the brush stroke
    pub color: Color
}

impl BrushProperties {
    ///
    /// Creates a new brush properties object with the settings at their defaults
    ///
    pub fn new() -> BrushProperties {
        BrushProperties {
            size:       5.0,
            opacity:    1.0,
            color:      Color::Rgba(0.0, 0.0, 0.0, 1.0)
        }
    }
}
