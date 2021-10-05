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
