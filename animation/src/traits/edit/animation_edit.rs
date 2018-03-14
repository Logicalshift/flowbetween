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

impl AnimationEdit {
    ///
    /// If this edit contains an unassigned element ID, calls the specified function to supply a new
    /// element ID. If the edit already has an ID, leaves it unchanged.
    /// 
    pub fn assign_element_id<AssignFn: FnOnce() -> i64>(self, assign_element_id: AssignFn) -> AnimationEdit {
        use self::AnimationEdit::*;

        match self {
            Layer(layer_id, layer_edit) => Layer(layer_id, layer_edit.assign_element_id(assign_element_id)),
            other                       => other
        }
    }
}