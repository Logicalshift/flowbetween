use flo_animation::*;

///
/// Action that updates the brush preview
/// 
#[derive(Debug)]
pub enum BrushPreviewAction {
    /// Clears any existing brush preview
    Clear,

    /// Unsets the brush preview properties (so the next brush stroke will always reload them)
    UnsetProperties,

    /// Specifies the layer whose brush preview is being edited
    Layer(u64),

    /// Sets the brush definition to use for the brush preview
    BrushDefinition(BrushDefinition, BrushDrawingStyle),

    /// Sets the brush properties to use for the brush preview
    BrushProperties(BrushProperties),

    /// Adds a raw point to the brush preview
    AddPoint(RawPoint),

    /// Commits the brush preview to the current layer
    Commit
}
