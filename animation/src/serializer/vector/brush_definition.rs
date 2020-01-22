use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl BrushDefinitionElement {
    ///
    /// Generates a serialized version of this brush definition element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        self.definition().serialize(data);
        self.drawing_style().serialize(data);
    }

    ///
    /// Deserializes a brush definition element from the specified data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<BrushDefinitionElement> {
        match data.next_small_u64() {
            0 => {
                BrushDefinition::deserialize(data)
                    .and_then(move |defn| BrushDrawingStyle::deserialize(data)
                        .map(move |drawing_style| BrushDefinitionElement::new(element_id, defn, drawing_style)))
            }

            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn defn_element() {
        let mut encoded = String::new();
        let element     = BrushDefinitionElement::new(ElementId::Assigned(1), BrushDefinition::Simple, BrushDrawingStyle::Erase);
        element.serialize(&mut encoded);

        let decoded     = BrushDefinitionElement::deserialize(ElementId::Assigned(1), &mut encoded.chars());
        let decoded     = decoded.unwrap();

        assert!(decoded.definition() == element.definition());
        assert!(decoded.drawing_style() == element.drawing_style());
    }
}