use super::super::source::*;
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

    ///
    /// Deserializes a time point from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> TimePoint {
        TimePoint(data.next_f32(), data.next_f32(), data.next_f32())
    }

    ///
    /// Deserializes a time point from a data source, where the last point is known
    ///
    pub fn deserialize_next<Src: AnimationDataSource>(last: &TimePoint, data: &mut Src) -> TimePoint {
        TimePoint(data.next_f64_offset(last.0 as f64) as f32, data.next_f64_offset(last.1 as f64) as f32, data.next_f64_offset(last.2 as f64) as f32)
    }
}
