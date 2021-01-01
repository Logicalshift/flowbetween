use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::str::{Chars};

impl Shape {
    ///
    /// Generates a serialized version of this shape on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        match self {
            Shape::Circle { center, point }         => { data.write_chr('c'); data.write_f32(center.0 as f32); data.write_f32(center.1 as f32); data.write_f32(point.0 as f32); data.write_f32(point.1 as f32); },
            Shape::Rectangle { center, point }      => { data.write_chr('r'); data.write_f32(center.0 as f32); data.write_f32(center.1 as f32); data.write_f32(point.0 as f32); data.write_f32(point.1 as f32); },
            Shape::Polygon { sides, center, point } => { data.write_chr('p'); data.write_usize(*sides); data.write_f32(center.0 as f32); data.write_f32(center.1 as f32); data.write_f32(point.0 as f32); data.write_f32(point.1 as f32); }
        }
    }

    ///
    /// Deserializes a shape from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<Shape> {
        match data.next_small_u64() {
            0 => { 
                match data.next_chr() {
                    'c'     => { 
                        let center  = (data.next_f32() as f64, data.next_f32() as f64);
                        let point   = (data.next_f32() as f64, data.next_f32() as f64);

                        Some(Shape::Circle { center, point })
                    }

                    'r'     => { 
                        let center  = (data.next_f32() as f64, data.next_f32() as f64);
                        let point   = (data.next_f32() as f64, data.next_f32() as f64);

                        Some(Shape::Rectangle { center, point })
                    }

                    'p'     => { 
                        let sides   = data.next_usize();
                        let center  = (data.next_f32() as f64, data.next_f32() as f64);
                        let point   = (data.next_f32() as f64, data.next_f32() as f64);

                        Some(Shape::Polygon { sides, center, point })
                    }

                    _       => None
                }
            }
            
            _ => { None }
        }
    }
}

impl ShapeElement {
    ///
    /// Generates a serialized version of this shape element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        data.write_f32(self.width() as f32);
        self.shape().serialize(data);
    }

    ///
    /// Deserializes a shape element from a data source
    ///
    pub fn deserialize(element_id: ElementId, data: &mut Chars) -> Option<ShapeElement> {
        match data.next_small_u64() {
            0 => { 
                let width = data.next_f32() as f64;
                let shape = Shape::deserialize(data)?;

                Some(ShapeElement::new(element_id, width, shape))
            }

            _ => { None }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn circle() {
        let shape           = ShapeElement::circle(ElementId::Assigned(42), (110.0, 120.0), (310.0, 320.0));
        let mut encoded     = String::new();
        shape.serialize(&mut encoded);

        let decoded         = ShapeElement::deserialize(ElementId::Assigned(42), &mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == shape);
    }

    #[test]
    fn rectangle() {
        let shape           = ShapeElement::rectangle(ElementId::Assigned(42), (110.0, 120.0), (310.0, 320.0));
        let mut encoded     = String::new();
        shape.serialize(&mut encoded);

        let decoded         = ShapeElement::deserialize(ElementId::Assigned(42), &mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == shape);
    }

    #[test]
    fn polygon() {
        let shape           = ShapeElement::polygon(ElementId::Assigned(42), (110.0, 120.0), (310.0, 320.0), 6);
        let mut encoded     = String::new();
        shape.serialize(&mut encoded);

        let decoded         = ShapeElement::deserialize(ElementId::Assigned(42), &mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == shape);
    }
}
