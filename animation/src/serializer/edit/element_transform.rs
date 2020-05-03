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

            Move(x, y) => {
                data.write_chr('m');
                data.write_f64(*x);
                data.write_f64(*y);
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

                Some(ElementTransform::Move(x, y))
            }

            _ => None
        }
    }
}
