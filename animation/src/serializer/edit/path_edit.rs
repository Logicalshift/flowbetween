use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

impl PathEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PathEdit::*;

        match self {
            SelectBrush(elem, defn, style)  => { data.write_chr('S'); elem.serialize(data); defn.serialize(data); style.serialize(data); }
            BrushProperties(elem, props)    => { data.write_chr('P'); elem.serialize(data); props.serialize(data); }

            CreatePath(elem, components)    => { 
                data.write_chr('+'); 
                elem.serialize(data); 
                data.write_usize(components.len()); 

                let mut last_point = PathPoint::new(0.0, 0.0);
                for component in components.iter() {
                    last_point = component.serialize_next(&last_point, data);
                }
            },
        }
    }

    ///
    /// Reads a path edit from the specified data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<PathEdit> {
        match data.next_chr() {
            'S' => {
                ElementId::deserialize(data)
                    .and_then(|elem| BrushDefinition::deserialize(data)
                        .and_then(move |defn| BrushDrawingStyle::deserialize(data)
                            .map(move |style| PathEdit::SelectBrush(elem, defn, style))))
            }

            'P' => {
                ElementId::deserialize(data)
                    .and_then(|elem| BrushProperties::deserialize(data)
                        .map(move |props| PathEdit::BrushProperties(elem, props)))
            }

            '+' => {
                ElementId::deserialize(data)
                    .and_then(|elem| {
                        let num_components  = data.next_usize();
                        let mut components  = vec![];

                        let mut last_point  = PathPoint::new(0.0, 0.0);
                        for _ in 0..num_components {
                            if let Some((next_component, next_point)) = PathComponent::deserialize_next(&last_point, data) {
                                components.push(next_component);
                                last_point = next_point;
                            } else {
                                return None;
                            }
                        }

                        Some(PathEdit::CreatePath(elem, Arc::new(components)))
                    })
            }

            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn select_brush() {
        let mut encoded = String::new();
        PathEdit::SelectBrush(ElementId::Assigned(42), BrushDefinition::Simple, BrushDrawingStyle::Erase).serialize(&mut encoded);

        assert!(PathEdit::deserialize(&mut encoded.chars()) == Some(PathEdit::SelectBrush(ElementId::Assigned(42), BrushDefinition::Simple, BrushDrawingStyle::Erase)));
    }

    #[test]
    fn brush_properties() {
        let mut encoded = String::new();
        PathEdit::BrushProperties(ElementId::Assigned(42), BrushProperties::new()).serialize(&mut encoded);

        assert!(PathEdit::deserialize(&mut encoded.chars()) == Some(PathEdit::BrushProperties(ElementId::Assigned(42), BrushProperties::new())));
    }

    #[test]
    fn create_path() {
        let mut encoded = String::new();
        let edit        = PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]));
        edit.serialize(&mut encoded);

        assert!(PathEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }
}
