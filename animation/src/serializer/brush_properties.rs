use super::color::*;
use super::target::*;
use super::super::traits::*;

impl BrushProperties {
    ///
    /// Generates a serialized version of these brush properties on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        data.write_f32(self.size);
        data.write_f32(self.opacity);
        serialize_color(&self.color, data);
    }
}

