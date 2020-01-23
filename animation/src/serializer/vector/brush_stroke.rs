use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

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

        data.write_usize(self.points().len());
        for point in self.points().iter() {
            last_point = point.serialize_next(&last_point, data);
        }
    }

    ///
    /// Deserializes a brush stroke element from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<BrushElement> {
        match data.next_small_u64() {
            0 => {
                let num_points      = data.next_usize();
                let mut points      = vec![];

                let mut last_point  = BrushPoint { 
                    position:   (0.0, 0.0), 
                    cp1:        (0.0, 0.0), 
                    cp2:        (0.0, 0.0),
                    width:      0.0
                };

                for _ in 0..num_points {
                    let next_point = BrushPoint::deserialize_next(&last_point, data);
                    points.push(next_point);
                    last_point = next_point;
                }

                Some(BrushElement::new(element_id, Arc::new(points)))
            }

            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn brush_stroke() {
        let mut encoded = String::new();
        let element     = BrushElement::new(ElementId::Assigned(1), Arc::new(vec![
            BrushPoint { position: (1.0, 2.0), cp1: (3.0, 4.0), cp2: (5.0, 6.0), width: 7.0 },
            BrushPoint { position: (8.0, 9.0), cp1: (10.0, 11.0), cp2: (12.0, 13.0), width: 14.0 },
            BrushPoint { position: (15.0, 16.0), cp1: (17.0, 18.0), cp2: (19.0, 6.0), width: 20.0 },
            BrushPoint { position: (1.0, 2.0), cp1: (3.0, 4.0), cp2: (5.0, 6.0), width: 7.0 }
        ]));
        element.serialize(&mut encoded);

        let decoded     = BrushElement::deserialize(ElementId::Assigned(1), &mut encoded.chars());
        let decoded     = decoded.unwrap();

        assert!(decoded.points() == element.points());
    }
}