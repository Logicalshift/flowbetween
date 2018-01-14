use super::*;
use super::super::super::brushes::*;

impl VectorLayer for VectorLayerCore {
    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, when: Duration, new_element: Vector) {
        if let Some(keyframe) = self.find_nearest_keyframe(when) {
            let when = when - keyframe.start_time();

            keyframe.add_element(when, new_element);
        }
    }

    ///
    /// Finds the brush that will be used with the next element added to this layer
    /// 
    fn active_brush(&self, when: Duration) -> Arc<Brush> {
        if let Some(keyframe) = self.find_nearest_keyframe(when) {
            keyframe.active_properties().brush.clone()
        } else {
            create_brush_from_definition(&BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw)
        }
    }
}
