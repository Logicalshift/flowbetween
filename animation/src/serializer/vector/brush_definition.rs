use super::super::target::*;
use super::super::super::traits::*;

impl BrushDefinitionElement {
    ///
    /// Generates a serialized version of this brush definition element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        self.definition().serialize(data);
        self.drawing_style().serialize(data);
    }
}
