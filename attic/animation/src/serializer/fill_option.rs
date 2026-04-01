use super::source::*;
use super::target::*;
use super::super::traits::*;

impl FillOption {
    ///
    /// Generates a serialized version of this fill option on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::FillOption::*;
        use self::FillAlgorithm::*;
        use self::FillPosition::*;

        match self {
            RayCastDistance(distance)   => { data.write_chr('d'); data.write_f64(*distance); }
            MinGap(gap_length)          => { data.write_chr('g'); data.write_f64(*gap_length); }
            Position(InFront)           => { data.write_chr('F'); }
            Position(Behind)            => { data.write_chr('B'); }
            Algorithm(Convex)           => { data.write_chr('v'); }
            Algorithm(Concave)          => { data.write_chr('c'); }
            FitPrecision(precision)     => { data.write_chr('P'); data.write_f64(*precision); }
        }
    }

    ///
    /// Deserializes this fill option from a source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<FillOption> {
        match data.next_chr() {
            'd' => Some(FillOption::RayCastDistance(data.next_f64())),
            'g' => Some(FillOption::MinGap(data.next_f64())),
            'F' => Some(FillOption::Position(FillPosition::InFront)),
            'B' => Some(FillOption::Position(FillPosition::Behind)),
            'v' => Some(FillOption::Algorithm(FillAlgorithm::Convex)),
            'c' => Some(FillOption::Algorithm(FillAlgorithm::Concave)),
            'P' => Some(FillOption::FitPrecision(data.next_f64())),
            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ray_cast_distance() {
        let mut encoded = String::new();
        FillOption::RayCastDistance(32.0).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::RayCastDistance(32.0)));
    }

    #[test]
    fn min_gap() {
        let mut encoded = String::new();
        FillOption::MinGap(24.0).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::MinGap(24.0)));
    }

    #[test]
    fn fit_precision() {
        let mut encoded = String::new();
        FillOption::FitPrecision(1.5).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::FitPrecision(1.5)));
    }

    #[test]
    fn fill_in_front() {
        let mut encoded = String::new();
        FillOption::Position(FillPosition::InFront).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Position(FillPosition::InFront)));
    }

    #[test]
    fn fill_behind() {
        let mut encoded = String::new();
        FillOption::Position(FillPosition::Behind).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Position(FillPosition::Behind)));
    }

    #[test]
    fn concave() {
        let mut encoded = String::new();
        FillOption::Algorithm(FillAlgorithm::Concave).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Algorithm(FillAlgorithm::Concave)));
    }

    #[test]
    fn convex() {
        let mut encoded = String::new();
        FillOption::Algorithm(FillAlgorithm::Convex).serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Algorithm(FillAlgorithm::Convex)));
    }
}
