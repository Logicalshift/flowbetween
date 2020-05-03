use super::super::source::*;
use super::super::target::*;

use crate::traits::*;

impl Transformation {
    ///
    /// Generates a serialized version of this transformation on a data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::Transformation::*;

        match self {
            Matrix(matrix) => {
                data.write_chr('M');
                for x in 0..3 {
                    for y in 0..3 {
                        data.write_f64(matrix[x][y])
                    }
                }
            }
        }
    }

    ///
    /// Deserializes a transformation from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<Transformation> {
        match data.next_chr() {
            'M' => {
                let mut matrix = [[0.0; 3]; 3];

                for x in 0..3 {
                    for y in 0..3 {
                        matrix[x][y] = data.next_f64();
                    }
                }

                Some(Transformation::Matrix(matrix))
            }

            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matrix() {
        let transformation  = Transformation::Matrix([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }
}
