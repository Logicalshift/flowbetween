use super::layer_edit::*;

///
/// Represents an edit to an animation object
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum AnimationEdit {
    /// Edit to an existing layer
    Layer(u64, LayerEdit),

    /// Sets the canvas size for this animation
    SetSize(f64, f64),

    /// Adds a new layer and assigns it the specified ID
    /// Has no effect if a layer with that ID already exists
    AddNewLayer(u64),

    /// Removes the layer with the specified ID
    RemoveLayer(u64)
}
