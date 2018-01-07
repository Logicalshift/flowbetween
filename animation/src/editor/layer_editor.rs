use super::super::traits::*;

use std::time::Duration;

///
/// Performs edits on a layer
/// 
pub struct LayerEditor {
}

impl LayerEditor {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new() -> LayerEditor {
        LayerEditor { }
    }

    ///
    /// Performs a paiting action
    /// 
    fn paint(&self, target: &mut Layer, when: Duration, paint: PaintEdit) {
        use PaintEdit::*;

        match paint {
            SelectBrush(definition, draw_style) => {
                target.edit_vectors().unwrap()
                    .add_element(when, Box::new(BrushDefinitionElement::new(definition, draw_style)));
            },

            BrushProperties(new_properties)     => {
                target.edit_vectors().unwrap()
                    .add_element(when, Box::new(BrushPropertiesElement::new(new_properties)));
            },
            
            BrushStroke(points)                 => {
                target.edit_vectors().unwrap()
                    .add_element(when, Box::new(BrushElement::new(points)));
            }
        }
    }

    ///
    /// Performs some edits on the specified layer
    /// 
    pub fn perform<Edits: IntoIterator<Item=LayerEdit>>(&self, target: &mut Layer, edits: Edits) {
        use LayerEdit::*;

        for edit in edits {
            match edit {
                Paint(when, edit)           => self.paint(target, when, edit),

                AddKeyFrame(when)           => target.add_key_frame(when),
                RemoveKeyFrame(when)        => target.remove_key_frame(when)
            }
        }
    }
}
