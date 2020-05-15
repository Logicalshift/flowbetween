use super::super::source::*;
use super::super::target::*;

use crate::traits::*;

impl ElementTransform {
    ///
    /// Generates a serialized version of this transformation on a data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementTransform::*;

        match self {
            SetAnchor(x, y) => { 
                data.write_chr('A');
                data.write_f64(*x);
                data.write_f64(*y);
            }

            MoveTo(x, y) => {
                data.write_chr('m');
                data.write_f64(*x);
                data.write_f64(*y);
            }

            Align(alignment) => {
                data.write_chr('a');
                alignment.serialize(data);
            }

            FlipHorizontal => {
                data.write_chr('f');
                data.write_chr('h');
            }

            FlipVertical => {
                data.write_chr('f');
                data.write_chr('v');
            }

            Scale(x, y) => {
                data.write_chr('s');
                data.write_f64(*x);
                data.write_f64(*y);
            }

            Rotate(angle) => {
                data.write_chr('r');
                data.write_f64(*angle);
            }
        }
    }

    ///
    /// Reads an element transformation from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<ElementTransform> {
        match data.next_chr() {
            'A' => {
                let (x, y) = (data.next_f64(), data.next_f64());

                Some(ElementTransform::SetAnchor(x, y))
            }

            'm' => {
                let (x, y) = (data.next_f64(), data.next_f64());

                Some(ElementTransform::MoveTo(x, y))
            }

            'a' => {
                ElementAlign::deserialize(data)
                    .map(|align| ElementTransform::Align(align))
            }

            'f' => {
                match data.next_chr() {
                    'h' => Some(ElementTransform::FlipHorizontal),
                    'v' => Some(ElementTransform::FlipVertical),
                    _ => None
                }
            }

            's' => {
                let (x, y) = (data.next_f64(), data.next_f64());

                Some(ElementTransform::Scale(x, y))
            }

            'r' => {
                let angle = data.next_f64();

                Some(ElementTransform::Rotate(angle))
            }

            _ => None
        }
    }
}
