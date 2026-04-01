use super::super::serializer::*;

use std::i64;

///
/// Storage/serialization structure used to represent the properties of a layer
///
pub struct LayerProperties {
    /// The name of this layer
    pub name: String,

    /// The alpha blending factor for this layer
    pub alpha: f64,

    /// The ordering of this layer, relative to other layers
    pub ordering: i64
}


impl Default for LayerProperties {
    fn default() -> LayerProperties {
        LayerProperties {
            name:       "".to_string(),
            alpha:      1.0,
            ordering:   i64::max_value()
        }
    }
}

impl LayerProperties {
    ///
    /// Serializes these file properties to a target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // Version 1 of the properties
        data.write_small_u64(1);

        data.write_str(&self.name);
        data.write_f64(self.alpha);
        data.write_i64(self.ordering);
    }

    ///
    /// Deserializes file properties from a target
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<LayerProperties> {
        let mut result = LayerProperties::default();

        match data.next_small_u64() {
            0 => {
                result.name     = data.next_string();
                result.alpha    = 1.0;
                result.ordering = data.next_i64();

                Some(result)
            },

            1 => {
                result.name     = data.next_string();
                result.alpha    = data.next_f64();
                result.ordering = data.next_i64();

                Some(result)
            }

            _ => None
        }
    }
}
