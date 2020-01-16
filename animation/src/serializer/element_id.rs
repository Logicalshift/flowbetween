use super::target::*;
use super::super::traits::*;

impl ElementId {
    ///
    /// Generates a serialized version of this element ID on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementId::*;

        match self {
            Unassigned      => { data.write_chr('?'); }
            Assigned(val)   => { 
                if val >= &0 {
                    data.write_chr('+'); data.write_small_u64((*val) as u64); 
                } else {
                    data.write_chr('-'); data.write_i64(*val);
                }
            }
        }
    }
}
