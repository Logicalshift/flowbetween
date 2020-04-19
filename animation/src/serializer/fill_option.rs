use super::source::*;
use super::target::*;
use super::super::traits::*;

impl FillOption {
    ///
    /// Generates a serialized version of this fill option on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::FillOption::*;

        match self {
            RayCastDistance(distance)   => { data.write_chr('d'); data.write_f64(*distance); }
            MinGap(gap_length)          => { data.write_chr('g'); data.write_f64(*gap_length); }
            FillBehind                  => { data.write_chr('B'); }
            Convex                      => { data.write_chr('v'); }
            Concave                     => { data.write_chr('c'); }
        }
    }

    ///
    /// Deserializes this fill option from a source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<FillOption> {
        match data.next_chr() {
            'd' => Some(FillOption::RayCastDistance(data.next_f64())),
            'g' => Some(FillOption::MinGap(data.next_f64())),
            'B' => Some(FillOption::FillBehind),
            'v' => Some(FillOption::Convex),
            'c' => Some(FillOption::Concave),
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
    fn fill_behind() {
        let mut encoded = String::new();
        FillOption::FillBehind.serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::FillBehind));
    }

    #[test]
    fn concave() {
        let mut encoded = String::new();
        FillOption::Concave.serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Concave));
    }

    #[test]
    fn convex() {
        let mut encoded = String::new();
        FillOption::Convex.serialize(&mut encoded);

        assert!(FillOption::deserialize(&mut encoded.chars()) == Some(FillOption::Convex));
    }
}
