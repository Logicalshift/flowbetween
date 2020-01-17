use super::super::target::*;
use super::super::super::traits::*;

impl TimeControlPoint {
    ///
    /// Generates a serialized version of this time control point on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        self.point.serialize(data);
        self.past.serialize(data);
        self.future.serialize(data);
    }

    ///
    /// Generates a serialized version of this time control point on the specified data target
    ///
    pub fn serialize_next<Tgt: AnimationDataTarget>(&self, last: &TimePoint, data: &mut Tgt) -> TimePoint {
        self.past.serialize_next(last, data);
        self.point.serialize_next(&self.past, data);
        self.future.serialize_next(&self.point, data);

        self.future.clone()
    }
}
