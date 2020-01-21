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
                        let components      = (0..num_components).into_iter()
                            .map(|_idx| PathComponent::deserialize(data))
                            .collect::<Option<Vec<_>>>();

                        components.map(move |components|
                            PathEdit::CreatePath(elem, Arc::new(components)))
                    })
            }

            _   => None
        }
    }
}
