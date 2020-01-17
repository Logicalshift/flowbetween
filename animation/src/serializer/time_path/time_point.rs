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

    ///
    /// Generates a serialized version of this time point on the specified data target
    ///
    pub fn serialize_next<Tgt: AnimationDataTarget>(&self, last: &TimePoint, data: &mut Tgt) {
        data.write_next_f64(last.0 as f64, self.0 as f64);
        data.write_next_f64(last.1 as f64, self.1 as f64);
        data.write_next_f64(last.2 as f64, self.2 as f64);
    }
}
