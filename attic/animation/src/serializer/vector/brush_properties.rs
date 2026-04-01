use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl BrushPropertiesElement {
    ///
    /// Generates a serialized version of this brush properties element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        self.brush_properties().serialize(data);
    }

    ///
    /// Deserializes a brush properties element from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<BrushPropertiesElement> {
        BrushProperties::deserialize(data)
            .map(move |props| BrushPropertiesElement::new(element_id, props))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn brush_properties_element() {
        let mut encoded = String::new();
        let element     = BrushPropertiesElement::new(ElementId::Assigned(1), BrushProperties::new());
        element.serialize(&mut encoded);

        let decoded     = BrushPropertiesElement::deserialize(ElementId::Assigned(1), &mut encoded.chars());
        let decoded     = decoded.unwrap();

        assert!(decoded.brush_properties() == element.brush_properties());
    }
}
