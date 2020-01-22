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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn in_front() {
        let mut encoded = String::new();
        ElementOrdering::InFront.serialize(&mut encoded);

        assert!(ElementOrdering::deserialize(&mut encoded.chars()) == Some(ElementOrdering::InFront));
    }

    #[test]
    fn behind() {
        let mut encoded = String::new();
        ElementOrdering::Behind.serialize(&mut encoded);

        assert!(ElementOrdering::deserialize(&mut encoded.chars()) == Some(ElementOrdering::Behind));
    }

    #[test]
    fn to_top() {
        let mut encoded = String::new();
        ElementOrdering::ToTop.serialize(&mut encoded);

        assert!(ElementOrdering::deserialize(&mut encoded.chars()) == Some(ElementOrdering::ToTop));
    }

    #[test]
    fn to_bottom() {
        let mut encoded = String::new();
        ElementOrdering::ToBottom.serialize(&mut encoded);

        assert!(ElementOrdering::deserialize(&mut encoded.chars()) == Some(ElementOrdering::ToBottom));
    }

    #[test]
    fn before() {
        let mut encoded = String::new();
        ElementOrdering::Before(ElementId::Assigned(42)).serialize(&mut encoded);

        assert!(ElementOrdering::deserialize(&mut encoded.chars()) == Some(ElementOrdering::Before(ElementId::Assigned(42))));
    }
}
