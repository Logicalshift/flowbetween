use super::layer_editor::*;
use super::super::traits::*;

///
/// Editor that can be used to commit edits to an animation
/// 
pub struct AnimationEditor {
    layer_editor: LayerEditor
}

impl AnimationEditor {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new() -> AnimationEditor {
        AnimationEditor {
            layer_editor: LayerEditor::new()
         }
    }

    ///
    /// Performs some edits on the specified editable animation
    /// 
    pub fn perform<Edits: IntoIterator<Item=AnimationEdit>>(&self, target: &mut MutableAnimation, edits: Edits) {
        use AnimationEdit::*;

        for edit in edits {
            match edit {
                SetSize(x, y)               => { target.set_size((x, y)); }
                AddNewLayer(layer_id)       => { target.add_layer(layer_id); },
                RemoveLayer(layer_id)       => { target.remove_layer(layer_id); },

                Layer(layer_id, layer_edit)   => {
                    if let Some(mut edit_layer) = target.edit_layer(layer_id) {
                        self.layer_editor.perform(&mut *edit_layer, vec![layer_edit]);
                    }
                },

                Element(element_id, when, element_edit) => {
                    target.edit_element(element_id, when, element_edit);
                }
            }
        }
    }
}
