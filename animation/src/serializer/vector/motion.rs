use super::super::target::*;
use super::super::super::traits::*;

impl MotionElement {
    ///
    /// Generates a serialized version of this motion element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        self.motion().serialize(data);
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
}
