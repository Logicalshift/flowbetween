use super::super::target::*;
use super::super::super::traits::*;

impl BrushPoint {
    ///
    /// Generates a serialized version of this brush point on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_f32(self.position.0); data.write_f32(self.position.1);
        data.write_f32(self.cp1.0);      data.write_f32(self.cp1.1);
        data.write_f32(self.cp2.0);      data.write_f32(self.cp2.0);
        data.write_f32(self.width);
    }

    ///
    /// Generates a serialized version of this brush point on the specified data target
    ///
    pub fn serialize_next<Tgt: AnimationDataTarget>(&self, last: &BrushPoint, data: &mut Tgt) -> BrushPoint {
        data.write_next_f64(last.position.0 as f64, self.position.0 as f64); data.write_next_f64(last.position.1 as f64, self.position.1 as f64);
        data.write_next_f64(last.cp1.0 as f64, self.cp1.0 as f64);           data.write_next_f64(last.cp1.1 as f64, self.cp1.1 as f64);
        data.write_next_f64(last.cp2.0 as f64, self.cp2.0 as f64);           data.write_next_f64(last.cp2.0 as f64, self.cp2.0 as f64);
        data.write_next_f64(last.width as f64, self.width as f64);

        self.clone()
    }
}
