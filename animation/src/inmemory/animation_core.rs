use super::vector_layer::*;
use super::super::traits::*;

use std::collections::HashMap;

///
/// The core in-memory animation data structure
/// 
pub struct AnimationCore {
    /// The edit log for this animation
    pub edit_log: Vec<AnimationEdit>,

    /// The next element ID to assign
    pub next_element_id: i64,

    /// The size of the animation canvas
    pub size: (f64, f64),

    /// The vector layers in this animation
    pub vector_layers: HashMap<u64, InMemoryVectorLayer>,

    /// The motions in this animation
    pub motions: Vec<(ElementId, Motion)>
}

impl AnimationCore {
    ///
    /// Performs a single edit on this core
    /// 
    pub fn edit(&mut self, edit: &AnimationEdit) {
        use self::AnimationEdit::*;

        match edit {
            SetSize(x, y) => { 
                self.size = (*x, *y); 
            },

            AddNewLayer(new_layer_id) => { 
                self.vector_layers.entry(*new_layer_id)
                    .or_insert_with(|| InMemoryVectorLayer::new(*new_layer_id));
            }

            RemoveLayer(old_layer_id) => {
                self.vector_layers.remove(old_layer_id);
            },

            Layer(layer_id, edit) => { 
                self.vector_layers.get(&layer_id)
                    .map(|layer| layer.edit(edit));
            },

            Motion(motion_id, edit) => {
                // TODO!
            },

            Element(element_id, when, element_edit) => {
                // We don't know which layer owns the element, so we just tell all of them to perform the edit (layers without the element will ignore the instruction)
                self.vector_layers.values()
                    .for_each(move |layer| layer.edit_element(*element_id, *when, element_edit));
            }
        }
    }
}