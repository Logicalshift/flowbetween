use super::super::source::*;
use super::super::target::*;
use crate::traits::*;
use crate::serializer::vector::{ResolveElements};

use serde_json as json;

use std::sync::*;
use std::str::{Chars};

impl LayerEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::LayerEdit::*;

        match self {
            Paint(when, edit)                                   => { data.write_chr('P'); data.write_duration(*when); edit.serialize(data); },
            Path(when, edit)                                    => { data.write_chr('p'); data.write_duration(*when); edit.serialize(data); },
            AddKeyFrame(when)                                   => { data.write_chr('+'); data.write_duration(*when); },
            RemoveKeyFrame(when)                                => { data.write_chr('-'); data.write_duration(*when); },
            SetName(name)                                       => { data.write_chr('N'); data.write_str(name); },
            SetOrdering(ordering)                               => { data.write_chr('O'); data.write_u64(*ordering); }
            SetAlpha(alpha)                                     => { data.write_chr('a'); data.write_f64(*alpha); }
            CreateAnimation(when, id, description)              => { data.write_chr('A'); data.write_duration(*when); id.serialize(data); data.write_str(&json::to_string(description).unwrap()); }
            CreateElement(when, id, vector)                     => { data.write_chr('V'); data.write_duration(*when); id.serialize(data); vector.serialize(data); },
            CreateElementUnattachedToFrame(when, id, vector)    => { data.write_chr('v'); data.write_duration(*when); id.serialize(data); vector.serialize(data); },

            Cut { path, when, inside_group }   => { 
                data.write_chr('c'); 
                data.write_duration(*when);
                inside_group.serialize(data);
                ElementId::Unassigned.serialize(data);

                data.write_usize(path.len()); 

                let mut last_point = PathPoint::new(0.0, 0.0);
                for component in path.iter() {
                    last_point = component.serialize_next(&last_point, data);
                }
            }
        }
    }

    ///
    /// Deserializes a layer edit from the supplied data source
    ///
    pub fn deserialize(data: &mut Chars) -> Option<LayerEdit> {
        match data.next_chr() {
            'P' => {
                let when = data.next_duration();
                PaintEdit::deserialize(data)
                    .map(move |edit| LayerEdit::Paint(when, edit))
            }
            'p' => { 
                let when = data.next_duration();
                PathEdit::deserialize(data)
                    .map(move |edit| LayerEdit::Path(when, edit))
            }
            '+' => { Some(LayerEdit::AddKeyFrame(data.next_duration())) }
            '-' => { Some(LayerEdit::RemoveKeyFrame(data.next_duration())) }
            'N' => { Some(LayerEdit::SetName(data.next_string())) }
            'O' => { Some(LayerEdit::SetOrdering(data.next_u64())) }
            'A' => { Some(LayerEdit::CreateAnimation(data.next_duration(), ElementId::deserialize(data)?, json::from_str(&data.next_string()).ok()?)) }
            'a' => { Some(LayerEdit::SetAlpha(data.next_f64())) }

            'V' => { 
                let when    = data.next_duration();
                let id      = ElementId::deserialize(data)?;
                let vector  = Vector::deserialize(id, data)?;
                let vector  = vector.resolve(&mut |_| None)?;

                Some(LayerEdit::CreateElement(when, id, vector)) 
            }

            'v' => { 
                let when    = data.next_duration();
                let id      = ElementId::deserialize(data)?;
                let vector  = Vector::deserialize(id, data)?;
                let vector  = vector.resolve(&mut |_| None)?;

                Some(LayerEdit::CreateElementUnattachedToFrame(when, id, vector)) 
            }

            'c' => {
                let when            = data.next_duration();
                let inside_group    = ElementId::deserialize(data)?;
                let _unused         = ElementId::deserialize(data)?;
                let path_len        = data.next_usize();

                let mut last_point  = PathPoint::new(0.0, 0.0);
                let mut components  = vec![];
                for _component_num in 0..path_len {
                    let (component, next_point) = PathComponent::deserialize_next(&last_point, data)?;
                    components.push(component);
                    last_point = next_point;
                }

                let path            = Arc::new(components);

                Some(LayerEdit::Cut { path, when, inside_group })
            }

            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration};
    use flo_curves::*;
    use flo_curves::arc::*;
    use flo_curves::bezier::path::*;
    use flo_canvas_animation::description::*;

    #[test]
    fn paint() {
        let mut encoded = String::new();
        let edit        = LayerEdit::Paint(Duration::from_millis(1234), PaintEdit::SelectBrush(ElementId::Assigned(42), BrushDefinition::Simple, BrushDrawingStyle::Erase));
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn path() {
        let mut encoded = String::new();
        let edit        = LayerEdit::Path(Duration::from_millis(1234), PathEdit::SelectBrush(ElementId::Assigned(42), BrushDefinition::Simple, BrushDrawingStyle::Erase));
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn add_key_frame() {
        let mut encoded = String::new();
        let edit        = LayerEdit::AddKeyFrame(Duration::from_millis(1234));
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn remove_key_frame() {
        let mut encoded = String::new();
        let edit        = LayerEdit::RemoveKeyFrame(Duration::from_millis(1234));
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn set_name() {
        let mut encoded = String::new();
        let edit        = LayerEdit::SetName("Test".to_string());
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn set_ordering() {
        let mut encoded = String::new();
        let edit        = LayerEdit::SetOrdering(42);
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn create_animation() {
        let mut encoded     = String::new();
        let circle          = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();
        let edit            = LayerEdit::CreateAnimation(Duration::from_millis(1234), ElementId::Assigned(42), RegionDescription(vec![circle.into()], EffectDescription::Sequence(vec![])));
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn cut() {
        let mut encoded = String::new();
        let edit        = LayerEdit::Cut { 
            path:           Arc::new(vec![PathComponent::Move(PathPoint::new(1.0, 2.0)), PathComponent::Line(PathPoint::new(2.0, 3.0)), PathComponent::Bezier(PathPoint::new(4.0, 5.0), PathPoint::new(6.0, 7.0), PathPoint::new(8.0, 9.0)), PathComponent::Close]),
            when:           Duration::from_millis(4200),
            inside_group:   ElementId::Assigned(1)
        };
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn set_alpha() {
        let mut encoded = String::new();
        let edit        = LayerEdit::SetAlpha(0.42);
        edit.serialize(&mut encoded);

        assert!(LayerEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }
}
