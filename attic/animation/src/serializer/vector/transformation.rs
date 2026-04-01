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

            Translate(dx, dy) => {
                data.write_chr('t');
                data.write_f64(*dx);
                data.write_f64(*dy);
            },

            FlipHoriz(x, y) => {
                data.write_chr('f');
                data.write_chr('h');
                data.write_f64(*x);
                data.write_f64(*y);
            }

            FlipVert(x, y) => {
                data.write_chr('f');
                data.write_chr('v');
                data.write_f64(*x);
                data.write_f64(*y);
            },

            Scale(ratiox, ratioy, (x, y)) => {
                data.write_chr('s');
                data.write_f64(*ratiox);
                data.write_f64(*ratioy);
                data.write_f64(*x);
                data.write_f64(*y);
            },

            Rotate(angle, (x, y)) => {
                data.write_chr('r');
                data.write_f64(*angle);
                data.write_f64(*x);
                data.write_f64(*y);
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

            't' => {
                let dx = data.next_f64();
                let dy = data.next_f64();

                Some(Transformation::Translate(dx, dy))
            }

            'f' => {
                match data.next_chr() {
                    'h' => {
                        let x = data.next_f64();
                        let y = data.next_f64();

                        Some(Transformation::FlipHoriz(x, y))
                    }

                    'v' => {
                        let x = data.next_f64();
                        let y = data.next_f64();

                        Some(Transformation::FlipVert(x, y))
                    }

                    _ => None
                }
            }

            's' => {
                let xratio  = data.next_f64();
                let yratio  = data.next_f64();
                let x       = data.next_f64();
                let y       = data.next_f64();

                Some(Transformation::Scale(xratio, yratio, (x, y)))
            }

            'r' => {
                let angle   = data.next_f64();
                let x       = data.next_f64();
                let y       = data.next_f64();

                Some(Transformation::Rotate(angle, (x, y)))
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

    #[test]
    fn translate() {
        let transformation  = Transformation::Translate(11.0, 12.0);
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }

    #[test]
    fn flip_horiz() {
        let transformation  = Transformation::FlipHoriz(11.0, 12.0);
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }

    #[test]
    fn flip_vert() {
        let transformation  = Transformation::FlipVert(11.0, 12.0);
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }

    #[test]
    fn scale() {
        let transformation  = Transformation::Scale(1.0, 2.0, (3.0, 4.0));
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }

    #[test]
    fn rotate() {
        let transformation  = Transformation::Rotate(1.0, (3.0, 4.0));
        let mut encoded     = String::new();
        transformation.serialize(&mut encoded);

        let decoded         = Transformation::deserialize(&mut encoded.chars());
        let decoded         = decoded.unwrap();

        assert!(decoded == transformation);
    }
}
