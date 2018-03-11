use super::super::traits::*;

use std::time::Duration;
use std::sync::*;

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
            SelectBrush(id, definition, draw_style) => {
                target.edit_vectors().unwrap()
                    .add_element(when, Vector::new(BrushDefinitionElement::new(definition, draw_style)));
            },

            BrushProperties(id, new_properties)     => {
                target.edit_vectors().unwrap()
                    .add_element(when, Vector::new(BrushPropertiesElement::new(new_properties)));
            },
            
            BrushStroke(id, points)                 => {
                let mut vectors     = target.edit_vectors().unwrap();
                let brush           = vectors.active_brush(when);

                let brush_points    = brush.brush_points_for_raw_points(&points);

                vectors.add_element(when, Vector::new(BrushElement::new(Arc::new(brush_points))));
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
