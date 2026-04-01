use super::source::*;
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

    ///
    /// Deserializes this drawing style from a source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<BrushDrawingStyle> {
        match data.next_chr() {
            '+' => Some(BrushDrawingStyle::Draw),
            '-' => Some(BrushDrawingStyle::Erase),
            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn draw() {
        let mut encoded = String::new();
        BrushDrawingStyle::Draw.serialize(&mut encoded);

        assert!(BrushDrawingStyle::deserialize(&mut encoded.chars()) == Some(BrushDrawingStyle::Draw));
    }

    #[test]
    fn deserialize() {
        assert!(BrushDrawingStyle::deserialize(&mut "+".chars()) == Some(BrushDrawingStyle::Draw));
        assert!(BrushDrawingStyle::deserialize(&mut "-".chars()) == Some(BrushDrawingStyle::Erase));
    }
}
