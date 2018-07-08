use super::*;
use super::super::super::traits::*;

use std::time::Duration;

impl VectorLayerCore {
    ///
    /// Adds a new key frame to this core 
    /// 
    pub fn add_key_frame(&mut self, time_offset: Duration) {
        // TODO: do nothing if the keyframe is already created

        // Generate a new keyframe
        let new_keyframe = VectorKeyFrame::new(time_offset, self.vector_map.clone());

        // Add in order to the existing keyframes
        self.keyframes.push(Arc::new(new_keyframe));
        self.sort_key_frames();
    }

    ///
    /// Removes a keyframe from this core
    /// 
    pub fn remove_key_frame(&mut self, time_offset: Duration) {
        // Binary search for the key frame
        let search_result = self.keyframes.binary_search_by(|a| a.start_time().cmp(&time_offset));

        // Remove only if we found an exact match
        if let Ok(frame_number) = search_result {
            self.keyframes.remove(frame_number);
        }
    }

    ///
    /// Adds a new vector element to this layer
    /// 
    pub fn add_element(&mut self, when: Duration, new_element: Vector) {
        if let Some(keyframe) = self.find_nearest_keyframe(when) {
            let when = when - keyframe.start_time();

            keyframe.add_element(when, new_element);
        }
    }

    ///
    /// Performs a paint edit on this layer
    /// 
    pub fn paint(&mut self, when: Duration, paint: &PaintEdit) {
        use self::PaintEdit::*;

        match paint {
            SelectBrush(id, definition, draw_style) => {
                let select_brush = Vector::new(BrushDefinitionElement::new(*id, definition.clone(), *draw_style));

                self.add_element(when, select_brush);
            },

            BrushProperties(id, new_properties)     => {
                let brush_properties = Vector::new(BrushPropertiesElement::new(*id, *new_properties));

                self.add_element(when, brush_properties);
            },
            
            BrushStroke(id, points)                 => {
                let brush           = self.active_brush(when);
                let brush_points    = brush.brush_points_for_raw_points(&points);

                let brush_stroke    = Vector::new(BrushElement::new(*id, Arc::new(brush_points)));

                self.add_element(when, brush_stroke);
            }
        }
    }

    ///
    /// Performs a layer edit on this layer
    /// 
    pub fn edit(&mut self, edit: &LayerEdit) {
        use self::LayerEdit::*;

        match edit {
            Paint(when, edit)           => self.paint(*when, edit),

            AddKeyFrame(when)           => self.add_key_frame(*when),
            RemoveKeyFrame(when)        => self.remove_key_frame(*when)
        }
    }

    ///
    /// Sets the control points for an element
    /// 
    pub fn set_control_points(&mut self, element_id: ElementId, control_points: &Vec<(f32, f32)>) {
        unimplemented!()
    }
}