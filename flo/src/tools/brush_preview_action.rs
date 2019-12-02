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

    /// If any elements overlap the brush preview, combine them into a single element
    CombineCollidingElements,

    /// Commits the brush preview to the current layer
    Commit,

    /// Commits the brush preview as a path to the current layer
    CommitAsPath
}
