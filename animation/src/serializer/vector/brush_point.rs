use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl BrushPoint {
    ///
    /// Generates a serialized version of this brush point on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_f32(self.position.0); data.write_f32(self.position.1);
        data.write_f32(self.cp1.0);      data.write_f32(self.cp1.1);
        data.write_f32(self.cp2.0);      data.write_f32(self.cp2.1);
        data.write_f32(self.width);
    }

    ///
    /// Generates a serialized version of this brush point on the specified data target
    ///
    pub fn serialize_next<Tgt: AnimationDataTarget>(&self, last: &BrushPoint, data: &mut Tgt) -> BrushPoint {
        data.write_next_f64(last.position.0 as f64, self.position.0 as f64); data.write_next_f64(last.position.1 as f64, self.position.1 as f64);
        data.write_next_f64(last.cp1.0 as f64, self.cp1.0 as f64);           data.write_next_f64(last.cp1.1 as f64, self.cp1.1 as f64);
        data.write_next_f64(last.cp2.0 as f64, self.cp2.0 as f64);           data.write_next_f64(last.cp2.1 as f64, self.cp2.1 as f64);
        data.write_next_f64(last.width as f64, self.width as f64);

        self.clone()
    }

    ///
    /// Deserializes a brush point from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> BrushPoint {
        let position    = (data.next_f32(), data.next_f32());
        let cp1         = (data.next_f32(), data.next_f32());
        let cp2         = (data.next_f32(), data.next_f32());
        let width       = data.next_f32();

        BrushPoint { 
            position, cp1, cp2, width
        }
    }

    ///
    /// Deserializes a brush point from a data source
    ///
    pub fn deserialize_next<Src: AnimationDataSource>(last: &BrushPoint, data: &mut Src) -> BrushPoint {
        let position    = (data.next_f64_offset(last.position.0 as f64), data.next_f64_offset(last.position.1 as f64));
        let cp1         = (data.next_f64_offset(last.cp1.0 as f64), data.next_f64_offset(last.cp1.1 as f64));
        let cp2         = (data.next_f64_offset(last.cp2.0 as f64), data.next_f64_offset(last.cp2.1 as f64));
        let width       = data.next_f64_offset(last.width as f64);

        let position    = (position.0 as f32, position.1 as f32);
        let cp1         = (cp1.0 as f32, cp1.1 as f32);
        let cp2         = (cp2.0 as f32, cp2.1 as f32);
        let width       = width as f32;

        BrushPoint { 
            position, cp1, cp2, width
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn brush_point() {
        let mut encoded = String::new();
        let element     = BrushPoint {
            position:   (1.0, 2.0),
            cp1:        (3.0, 4.0),
            cp2:        (5.0, 6.0),
            width:      7.0
        };
        element.serialize(&mut encoded);

        let decoded     = BrushPoint::deserialize(&mut encoded.chars());

        assert!(decoded == element);
    }

    #[test]
    fn brush_point_next() {
        let mut encoded = String::new();
        let last        = BrushPoint {
            position:   (1.0, 2.0),
            cp1:        (3.0, 4.0),
            cp2:        (5.0, 6.0),
            width:      7.0
        };
        let element     = BrushPoint {
            position:   (8.0, 9.0),
            cp1:        (10.0, 11.0),
            cp2:        (12.0, 13.0),
            width:      14.0
        };
        element.serialize_next(&last, &mut encoded);

        let decoded     = BrushPoint::deserialize_next(&last, &mut encoded.chars());

        assert!(decoded == element);
    }
}
