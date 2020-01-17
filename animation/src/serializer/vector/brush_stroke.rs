use super::super::target::*;
use super::super::super::traits::*;

impl BrushElement {
    ///
    /// Generates a serialized version of this brush stroke element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // Version 0
        data.write_small_u64(0);

        // Write the points
        let mut last_point = BrushPoint { 
            position:   (0.0, 0.0), 
            cp1:        (0.0, 0.0), 
            cp2:        (0.0, 0.0),
            width:      0.0
        };

        for point in self.points().iter() {
            last_point = point.serialize_next(&last_point, data);
        }
    }
}
