use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl ElementOrdering {
    ///
    /// Generates a serialized version of this ordering on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementOrdering::*;

        match self {
            InFront         => { data.write_chr('+'); }
            Behind          => { data.write_chr('-'); }
            ToTop           => { data.write_chr('^'); }
            ToBottom        => { data.write_chr('v'); }
            Before(elem)    => { data.write_chr('B'); elem.serialize(data); }
        }
    }

    ///
    /// Deserializes an ElementOrdering from the specified source stream
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<ElementOrdering> {
        use self::ElementOrdering::*;

        match data.next_chr() {
            '+' => Some(InFront),
            '-' => Some(Behind),
            '^' => Some(ToTop),
            'v' => Some(ToBottom),
            'B' => ElementId::deserialize(data).map(|elem| Before(elem)),
            _   => None
        }
    }
}
