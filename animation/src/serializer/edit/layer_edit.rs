use super::super::target::*;
use super::super::super::traits::*;

impl LayerEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::LayerEdit::*;

        match self {
            Paint(when, edit)       => { data.write_chr('P'); unimplemented!("Paint"); },
            Path(when, edit)        => { data.write_chr('p'); unimplemented!("Path"); },
            AddKeyFrame(when)       => { data.write_chr('+'); unimplemented!("AddKeyFrame"); },
            RemoveKeyFrame(when)    => { data.write_chr('-'); unimplemented!("RemoveKeyFrame"); },
            SetName(name)           => { data.write_chr('N'); unimplemented!("SetName"); },
            SetOrdering(ordering)   => { data.write_chr('O'); data.write_u32(*ordering); }
        }
    }
}
