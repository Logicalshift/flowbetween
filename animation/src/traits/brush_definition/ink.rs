
///
/// Ink brushes are solid lines of varying width. This defines how they behave.
/// The actual behaviour is implemented by the `InkBrush` structure.
/// 
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct InkDefinition {
    /// Width at pressure 0%
    pub min_width: f32,

    /// Width at pressure 100%
    pub max_width: f32,

    // Distance to scale up at the start of the brush stroke
    pub scale_up_distance: f32
}
