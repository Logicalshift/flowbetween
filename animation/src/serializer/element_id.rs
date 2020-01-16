use super::target::*;
use super::super::traits::*;

impl ElementId {
    ///
    /// Generates a serialized version of this element ID on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementId::*;

        match self {
            Assigned(val)   => { data.write_chr('e'); data.write_i64(*val); }
            Unassigned      => { data.write_chr('?'); }
        }
    }
}
