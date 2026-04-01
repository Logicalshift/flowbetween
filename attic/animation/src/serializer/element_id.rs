use super::source::*;
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

    ///
    /// Deserializes this element ID from a source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<ElementId> {
        match data.next_chr() {
            '?' => Some(ElementId::Unassigned),
            '+' => Some(ElementId::Assigned(data.next_small_u64() as i64)),
            '-' => Some(ElementId::Assigned(data.next_i64())),
            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unassigned() {
        let mut encoded = String::new();
        ElementId::Unassigned.serialize(&mut encoded);

        assert!(ElementId::deserialize(&mut encoded.chars()) == Some(ElementId::Unassigned));
    }

    #[test]
    fn assigned_1() {
        let mut encoded = String::new();
        ElementId::Assigned(42).serialize(&mut encoded);

        assert!(ElementId::deserialize(&mut encoded.chars()) == Some(ElementId::Assigned(42)));
    }

    #[test]
    fn assigned_2() {
        let mut encoded = String::new();
        ElementId::Assigned(-42).serialize(&mut encoded);

        assert!(ElementId::deserialize(&mut encoded.chars()) == Some(ElementId::Assigned(-42)));
    }

    #[test]
    fn assigned_3() {
        assert!(ElementId::deserialize(&mut "+qB".chars()) == Some(ElementId::Assigned(42)));
    }
}
