use crate::serializer::source::*;
use crate::serializer::target::*;
use crate::traits::*;

use serde_json as json;

impl AnimationElement {
    ///
    /// Generates a serialized version of this brush properties element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        let as_json = json::to_string(&self.description()).unwrap();
        data.write_str(&as_json);
    }

    ///
    /// Deserializes a brush properties element from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<AnimationElement> {
        let as_json     = data.next_string();
        let description = json::from_str(&as_json).ok()?;

        Some(AnimationElement::new(element_id, description))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use flo_curves::*;
    use flo_curves::arc::*;
    use flo_curves::bezier::path::*;
    use flo_canvas_animation::description::*;

    #[test]
    fn animation_element() {
        let circle              = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();
        let animation_region    = RegionDescription(vec![circle.into()], EffectDescription::Sequence(vec![]));

        let mut encoded = String::new();
        let element     = AnimationElement::new(ElementId::Assigned(1), animation_region);
        element.serialize(&mut encoded);

        let decoded     = AnimationElement::deserialize(ElementId::Assigned(1), &mut encoded.chars());
        let decoded     = decoded.unwrap();

        assert!(decoded.description() == element.description());
    }
}
