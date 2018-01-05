use super::layer_edit::*;

///
/// Represents an edit to an animation object
/// 
pub enum AnimationEdit {
    /// Edit to an existing layer
    Layer(LayerEdit)
}
