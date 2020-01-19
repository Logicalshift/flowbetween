use super::color::*;
use super::source::*;
use super::target::*;
use super::super::traits::*;

impl BrushProperties {
    ///
    /// Generates a serialized version of these brush properties on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        data.write_f32(self.size);
        data.write_f32(self.opacity);
        serialize_color(&self.color, data);
    }

    ///
    /// Deserializes brush properties from a stream
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<BrushProperties> {
        if data.next_small_u64() == 0 {
            let size    = data.next_f32();
            let opacity = data.next_f32();
            let color   = deserialize_color(data);

            color.map(|color| {
                BrushProperties {
                    size, opacity, color
                }
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use flo_canvas::*;

    #[test]
    fn brush_properties_1() {
        let mut encoded = String::new();
        BrushProperties { size: 20.0, opacity: 1.0, color: Color::Hsluv(0.2, 0.6, 0.4, 1.0) }.serialize(&mut encoded);

        assert!(BrushProperties::deserialize(&mut encoded.chars()) == Some(BrushProperties { size: 20.0, opacity: 1.0, color: Color::Hsluv(0.2, 0.6, 0.4, 1.0) }));
    }

    #[test]
    fn brush_properties_2() {
        assert!(BrushProperties::deserialize(&mut "AAAAoBBAAAg/AhzMTmZamZ//P".chars()) == Some(BrushProperties { size: 20.0, opacity: 1.0, color: Color::Hsluv(0.2, 0.6, 0.4, 1.0) }));
    }
}
