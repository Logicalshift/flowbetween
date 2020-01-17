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
}
