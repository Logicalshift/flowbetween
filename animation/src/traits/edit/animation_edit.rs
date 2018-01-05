use super::layer_edit::*;
use super::super::brush_definition::*;

///
/// Represents an edit to an animation object
/// 
pub enum AnimationEdit {
    /// Defines the brush with the specified ID.
    /// If a brush with this ID already exists, then it is left alone
    DefineBrush(u32, BrushDefinition),

    /// Edit to an existing layer
    Layer(LayerEdit)
}
