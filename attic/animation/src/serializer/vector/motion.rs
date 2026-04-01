use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

impl MotionElement {
    ///
    /// Generates a serialized version of this motion element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        self.motion().serialize(data);
    }

    ///
    /// Deserializes a MotionElement from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<MotionElement> {
        Motion::deserialize(data)
            .map(move |motion| MotionElement::new(element_id, motion))
    }
}

impl Motion {
    ///
    /// Generates a serialized version of this motion on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::Motion::*;

        match &*self {
            None                    => { data.write_chr('X'); }
            Reverse(motion)         => { data.write_chr('R'); motion.serialize(data); }
            Translate(translation)  => { data.write_chr('T'); translation.serialize(data); }
        }
    }

    ///
    /// Deserializes a Motion from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<Motion> {
        match data.next_chr() {
            'X' => Some(Motion::None),
            'R' => {
                let motion = Motion::deserialize(data);
                motion.map(|motion| Motion::Reverse(Arc::new(motion)))
            }
            'T' => {
                Some(Motion::Translate(TranslateMotion::deserialize(data)))
            }

            _ => None
        }
    }
}

impl TranslateMotion {
    ///
    /// Generates a serialized version of this translation on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_f32(self.origin.0);
        data.write_f32(self.origin.1);
        self.translate.serialize(data);
    }

    ///
    /// Deserializes a translate motion from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> TranslateMotion {
        TranslateMotion {
            origin:     (data.next_f32(), data.next_f32()),
            translate:  TimeCurve::deserialize(data)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration};

    #[test]
    fn translate_motion() {
        let motion = Motion::Translate(TranslateMotion {
            origin:     (2.0, 3.0),
            translate:  TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442)))
        });
        let motion = MotionElement::new(ElementId::Assigned(1), motion);

        let mut encoded = String::new();
        motion.serialize(&mut encoded);

        let decoded     = MotionElement::deserialize(ElementId::Assigned(1), &mut encoded.chars());
        let decoded     = decoded.unwrap();

        if let Motion::Translate(translate) = &*decoded.motion() {
            assert!(translate.origin == (2.0, 3.0));
            assert!(translate.translate.points.len() == 2);
            assert!(translate.translate.points[0].point.0 == 200.0);
            assert!(translate.translate.points[0].point.1 == 200.0);
        } else {
            assert!(false);
        }
    }
}
