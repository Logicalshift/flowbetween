use super::super::source::*;
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

    ///
    /// Deserializes a time control point from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> TimeControlPoint {
        let point   = TimePoint::deserialize(data);
        let past    = TimePoint::deserialize(data);
        let future  = TimePoint::deserialize(data);

        TimeControlPoint { point, past, future }
    }

    ///
    /// Deserializes a time control point from a data source, where the last point is known
    ///
    pub fn deserialize_next<Src: AnimationDataSource>(last: &TimePoint, data: &mut Src) -> (TimeControlPoint, TimePoint) {
        let past    = TimePoint::deserialize_next(last, data);
        let point   = TimePoint::deserialize_next(&past, data);
        let future  = TimePoint::deserialize_next(&point, data);

        (TimeControlPoint { point, past, future }, future)
    }
}
