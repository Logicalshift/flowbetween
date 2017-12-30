use canvas::*;

///
/// Standard properties for a brush stroke
/// 
/// These are the properties that are independent of the brush type.
/// Properties that define a brush can be found in brush_definition.
/// 
pub struct BrushProperties {
    /// The size of the brush stroke
    pub size: f32,

    /// The colour of the brush stroke
    pub color: Color
}
