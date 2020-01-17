use super::super::target::*;
use super::super::super::traits::*;

impl BrushPropertiesElement {
    ///
    /// Generates a serialized version of this brush properties element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        self.brush_properties().serialize(data);
    }
}
