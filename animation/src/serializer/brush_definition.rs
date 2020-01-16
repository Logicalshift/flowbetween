use super::target::*;
use super::super::traits::*;

impl BrushDefinition {
    ///
    /// Generates a serialized version of this brush definition on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::BrushDefinition::*;

        match self {
            Simple      => { data.write_chr('S'); }
            Ink(ink)    => { data.write_chr('I'); ink.serialize(data); }

        }
    }
}

impl InkDefinition {
    ///
    /// Generates a serialized version of this brush definition on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_small_u64(0);        // v0 definition

        data.write_f32(self.min_width);
        data.write_f32(self.max_width);
        data.write_f32(self.scale_up_distance);
    }
}
