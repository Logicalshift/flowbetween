use super::super::target::*;
use super::super::super::traits::*;

impl TimeCurve {
    ///
    /// Generates a serialized version of this time curve on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_usize(self.points.len());
        self.points.iter().for_each(|point| point.serialize(data));
    }
}
