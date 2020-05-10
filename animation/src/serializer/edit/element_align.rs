use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl ElementAlign {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementAlign::*;

        match self {
            Left    => data.write_chr('l'),
            Center  => data.write_chr('c'),
            Right   => data.write_chr('r'),

            Top     => data.write_chr('T'),
            Middle  => data.write_chr('M'),
            Bottom  => data.write_chr('B')
        }
    }

    ///
    /// Deserializes a motion edit from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<ElementAlign> {
        match data.next_chr() {
            'l'     => Some(ElementAlign::Left),
            'c'     => Some(ElementAlign::Center),
            'r'     => Some(ElementAlign::Right),

            'T'     => Some(ElementAlign::Top),
            'M'     => Some(ElementAlign::Middle),
            'B'     => Some(ElementAlign::Bottom),

            _       => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn left() {
        let mut encoded = String::new();
        ElementAlign::Left.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Left));
    }

    #[test]
    fn center() {
        let mut encoded = String::new();
        ElementAlign::Center.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Center));
    }

    #[test]
    fn right() {
        let mut encoded = String::new();
        ElementAlign::Right.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Right));
    }

    #[test]
    fn top() {
        let mut encoded = String::new();
        ElementAlign::Top.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Top));
    }

    #[test]
    fn middle() {
        let mut encoded = String::new();
        ElementAlign::Middle.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Middle));
    }

    #[test]
    fn bottom() {
        let mut encoded = String::new();
        ElementAlign::Bottom.serialize(&mut encoded);

        assert!(ElementAlign::deserialize(&mut encoded.chars()) == Some(ElementAlign::Bottom));
    }
}
