use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl MotionEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::MotionEdit::*;

        match self {
            Create                          => { data.write_chr('+'); }
            Delete                          => { data.write_chr('-'); }
            SetType(MotionType::None)       => { data.write_chr('T'); data.write_chr('-'); }
            SetType(MotionType::Reverse)    => { data.write_chr('T'); data.write_chr('R'); }
            SetType(MotionType::Translate)  => { data.write_chr('T'); data.write_chr('T'); }
            SetOrigin(x, y)                 => { data.write_chr('O'); data.write_f32(*x); data.write_f32(*y); }
            SetPath(curve)                  => { data.write_chr('P'); curve.serialize(data); }
        }
    }

    ///
    /// Deserializes a motion edit from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<MotionEdit> {
        match data.next_chr() {
            '+'     => Some(MotionEdit::Create),
            '-'     => Some(MotionEdit::Delete),
            'T'     => match data.next_chr() {
                '-' => Some(MotionEdit::SetType(MotionType::None)),
                'R' => Some(MotionEdit::SetType(MotionType::Reverse)),
                'T' => Some(MotionEdit::SetType(MotionType::Translate)),

                _   => None
            },
            'O'     => Some(MotionEdit::SetOrigin(data.next_f32(), data.next_f32())),
            'P'     => Some(MotionEdit::SetPath(TimeCurve::deserialize(data))),

            _       => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration};

    #[test]
    fn create() {
        let mut encoded = String::new();
        MotionEdit::Create.serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::Create));
    }

    #[test]
    fn delete() {
        let mut encoded = String::new();
        MotionEdit::Delete.serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::Delete));
    }

    #[test]
    fn set_type_none() {
        let mut encoded = String::new();
        MotionEdit::SetType(MotionType::None).serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::SetType(MotionType::None)));
    }

    #[test]
    fn set_type_reverse() {
        let mut encoded = String::new();
        MotionEdit::SetType(MotionType::Reverse).serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::SetType(MotionType::Reverse)));
    }

    #[test]
    fn set_type_translate() {
        let mut encoded = String::new();
        MotionEdit::SetType(MotionType::Translate).serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::SetType(MotionType::Translate)));
    }

    #[test]
    fn set_origin() {
        let mut encoded = String::new();
        MotionEdit::SetOrigin(10.0, 11.0).serialize(&mut encoded);

        assert!(MotionEdit::deserialize(&mut encoded.chars()) == Some(MotionEdit::SetOrigin(10.0, 11.0)));
    }

    #[test]
    fn set_path() {
        let mut encoded = String::new();
        let curve1      = TimeCurve::new(TimePoint::new(1.0, 2.0, Duration::from_millis(1000)), TimePoint::new(3.0, 4.0, Duration::from_millis(2000)));
        MotionEdit::SetPath(curve1.clone()).serialize(&mut encoded);

        if let Some(MotionEdit::SetPath(curve2)) = MotionEdit::deserialize(&mut encoded.chars()) {
            assert!(curve1.is_close_to(&curve2))
        } else {
            assert!(false);
        }
    }
}
