use super::super::target::*;
use super::super::super::traits::*;

impl TimePoint {
    ///
    /// Generates a serialized version of this time point on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_f32(self.0);
        data.write_f32(self.1);
        data.write_f32(self.2);
    }
}
