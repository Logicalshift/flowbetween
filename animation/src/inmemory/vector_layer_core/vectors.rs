use super::*;
use super::super::super::brushes::*;

impl VectorLayer for VectorLayerCore {
    ///
    /// Finds the brush that will be used with the next element added to this layer
    /// 
    fn active_brush(&self, when: Duration) -> Arc<dyn Brush> {
        if let Some(keyframe) = self.find_nearest_keyframe(when) {
            keyframe.active_properties().brush.clone()
        } else {
            create_brush_from_definition(&BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw)
        }
    }
}
