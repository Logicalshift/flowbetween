use super::super::target::*;
use super::super::super::traits::*;

impl LayerEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::LayerEdit::*;

        match self {
            Paint(when, edit)       => { data.write_chr('P'); data.write_duration(*when); edit.serialize(data); },
            Path(when, edit)        => { data.write_chr('p'); data.write_duration(*when); edit.serialize(data); },
            AddKeyFrame(when)       => { data.write_chr('+'); data.write_duration(*when); },
            RemoveKeyFrame(when)    => { data.write_chr('-'); data.write_duration(*when); },
            SetName(name)           => { data.write_chr('N'); data.write_str(name); },
            SetOrdering(ordering)   => { data.write_chr('O'); data.write_u32(*ordering); }
        }
    }
}
