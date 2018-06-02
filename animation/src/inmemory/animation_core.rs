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
    pub motions: HashMap<ElementId, Motion>,

    /// Maps element IDs to the attached motions
    pub motions_for_element: HashMap<ElementId, Vec<ElementId>>
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
                self.edit_motion(motion_id, edit);
            },

            Element(element_id, when, element_edit) => {
                // We don't know which layer owns the element, so we just tell all of them to perform the edit (layers without the element will ignore the instruction)
                self.vector_layers.values()
                    .for_each(move |layer| layer.edit_element(*element_id, *when, element_edit));
            }
        }
    }

    ///
    /// Performs a motion edit action
    /// 
    fn edit_motion(&mut self, motion_id: &ElementId, edit: &MotionEdit) {
        use self::MotionEdit::*;

        match edit {
            Create                  => { self.motions.insert(*motion_id, Motion::None); },
            Delete                  => { self.motions.remove(&motion_id); },
            SetType(motion_type)    => { self.motions.get_mut(&motion_id).map(|motion| motion.set_type(*motion_type)); },
            SetOrigin(x, y)         => { self.motions.get_mut(&motion_id).map(|motion| motion.set_origin((*x, *y))); },
            SetPath(path)           => { self.motions.get_mut(&motion_id).map(|motion| motion.set_path(path.clone())); },
            Attach(element_id)      => { self.motions_for_element.entry(*element_id).or_insert_with(|| vec![]).push(*motion_id); },
            Detach(element_id)      => { self.motions_for_element.get_mut(element_id).map(|motions| motions.retain(|element| element != motion_id)); }
        }
    }
}
