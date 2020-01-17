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
}
