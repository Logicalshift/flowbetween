use super::source::*;
use super::target::*;
use super::super::traits::*;

impl BrushDefinition {
    ///
    /// Generates a serialized version of this brush definition on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::BrushDefinition::*;

        match self {
            Simple      => { data.write_chr('S'); }
            Ink(ink)    => { data.write_chr('I'); ink.serialize(data); }
        }
    }

    ///
    /// Deserializes a brush definition from the specified data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<BrushDefinition> {
        match data.next_chr() {
            'S' => { Some(BrushDefinition::Simple) }
            'I' => { InkDefinition::deserialize(data).map(|ink| BrushDefinition::Ink(ink)) }
            _   => None
        }
    }
}

impl InkDefinition {
    ///
    /// Generates a serialized version of this brush definition on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_small_u64(0);        // v0 definition

        data.write_f32(self.min_width);
        data.write_f32(self.max_width);
        data.write_f32(self.scale_up_distance);
    }

    ///
    /// Deserializes an ink brush definition from the specified data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<InkDefinition> {
        match data.next_small_u64() {
            0 => {
                Some(InkDefinition {
                    min_width:          data.next_f32(),
                    max_width:          data.next_f32(),
                    scale_up_distance:  data.next_f32()
                })
            }
            _ => { None }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn brush_ink_defn_1() {
        let mut encoded = String::new();
        BrushDefinition::Ink(InkDefinition { min_width: 1.0, max_width: 2.0, scale_up_distance: 3.0 }).serialize(&mut encoded);

        assert!(BrushDefinition::deserialize(&mut encoded.chars()) == Some(BrushDefinition::Ink(InkDefinition { min_width: 1.0, max_width: 2.0, scale_up_distance: 3.0 })));
    }

    #[test]
    fn brush_ink_defn_2() {
        assert!(BrushDefinition::deserialize(&mut "IAAAAg/AAAAAABAAAQAB".chars()) == Some(BrushDefinition::Ink(InkDefinition { min_width: 1.0, max_width: 2.0, scale_up_distance: 3.0 })));
    }
}
