use animation::*;

///
/// Action that updates the brush preview
/// 
pub enum BrushPreviewAction {
    /// Clears any existing brush preview
    Clear,

    /// Sets the brush definition to use for the brush preview
    BrushDefinition(BrushDefinition, BrushDrawingStyle),

    /// Sets the brush properties to use for the brush preview
    BrushProperties(BrushProperties),

    /// Adds a raw point to the brush preview
    AddPoint(RawPoint),

    /// Commits the brush preview to the current layer
    Commit
}
