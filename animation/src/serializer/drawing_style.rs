use super::target::*;
use super::super::traits::*;

impl BrushDrawingStyle {
    ///
    /// Generates a serialized version of this drawing style on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::BrushDrawingStyle::*;

        match self {
            Draw    => { data.write_chr('+'); }
            Erase   => { data.write_chr('-'); }
        }
    }
}
