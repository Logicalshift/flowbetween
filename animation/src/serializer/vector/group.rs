use super::super::target::*;
use super::super::super::traits::*;

impl GroupElement {
    ///
    /// Generates a serialized version of this group element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        use self::GroupType::*;
        match self.group_type() {
            Normal      => { data.write_chr('N'); }
            Added       => { data.write_chr('+'); }
        }

        // Grouped elements
        data.write_usize(self.num_elements());
        for elem in self.elements() {
            // Write out the ID of this elmeent
            elem.id().serialize(data);

            // Serialize the element if it has no ID (ie, not a reference to another element)
            // Elements with IDs are expected to be found elsewhere
            if elem.id().is_unassigned() {
                elem.serialize(data);
            }
        }
        
        // Hint path, if one is set
        if let Some(hint_path) = self.hint_path() {
            data.write_chr('H');
            data.write_usize(hint_path.len());
            hint_path.iter().for_each(|path| path.serialize(data));
        } else {
            data.write_chr('X');
        }
    }
}
