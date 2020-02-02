use super::super::serializer::*;

///
/// Storage/serialization structure used to represent the properties of a layer
///
pub struct LayerProperties {
    /// The name of this layer
    pub name: String
}


impl Default for LayerProperties {
    fn default() -> LayerProperties {
        LayerProperties {
            name: "".to_string()
        }
    }
}

impl LayerProperties {
    ///
    /// Serializes these file properties to a target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // Version 0 of the properties
        data.write_small_u64(0);

        data.write_str(&self.name);
    }

    ///
    /// Deserializes file properties from a target
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<LayerProperties> {
        let mut result = LayerProperties::default();

        match data.next_small_u64() {
            0 => {
                result.name             = data.next_string();

                Some(result)
            }

            _ => None
        }
    }
}
