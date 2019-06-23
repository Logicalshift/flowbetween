///
/// How brush strokes are added to a frame
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BrushModificationMode {
    /// Brush strokes are added to existing paths where possible (to form larger paths)
    Additive,

    /// Brush strokes are each stored as individual objects
    Individual
}

///
/// The representation used to add new brush strokes to a frame
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BrushRepresentation {
    /// Brush strokes are stored as brush strokes (paths are procedurally generated)
    BrushStroke,

    /// Brush strokes are stored directly as paths (in additive mode, things the brush stroke is added to will also be turned to paths)
    Path
}
