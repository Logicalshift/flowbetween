use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

impl ElementEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementEdit::*;

        match self {
            AddAttachment(elem)         => { data.write_chr('+'); elem.serialize(data); }
            RemoveAttachment(elem)      => { data.write_chr('-'); elem.serialize(data); }
            Order(ordering)             => { data.write_chr('O'); ordering.serialize(data); }
            Delete                      => { data.write_chr('X'); }
            DetachFromFrame             => { data.write_chr('D'); }

            SetControlPoints(points)    => { 
                data.write_chr('C'); 
                data.write_usize(points.len());

                let mut last_point = (0.0f32, 0.0f32);
                for (x, y) in points.iter() {
                    data.write_next_f64(last_point.0 as f64, *x as f64);
                    data.write_next_f64(last_point.1 as f64, *y as f64);

                    last_point = (*x, *y);
                }
            }

            SetPath(path_components)    => { 
                data.write_chr('P'); 
                data.write_usize(path_components.len()); 

                let mut last_point = PathPoint::new(0.0, 0.0);
                for component in path_components.iter() {
                    last_point = component.serialize_next(&last_point, data);
                }
            }
        }
    }

    ///
    /// Deserializes an element edit from the specified source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<ElementEdit> {
        match data.next_chr() {
            '+' => {
                ElementId::deserialize(data)
                    .map(|elem| ElementEdit::AddAttachment(elem))
            }

            '-' => {
                ElementId::deserialize(data)
                    .map(|elem| ElementEdit::RemoveAttachment(elem))
            }

            'O' => {
                ElementOrdering::deserialize(data)
                    .map(|ordering| ElementEdit::Order(ordering))
            }

            'X' => {
                Some(ElementEdit::Delete)
            }

            'D' => {
                Some(ElementEdit::DetachFromFrame)
            }

            'C' => {
                let num_points      = data.next_usize();
                let mut last_point  = (0.0, 0.0);
                let mut points      = vec![];

                for _ in 0..num_points {
                    let x = data.next_f64_offset(last_point.0);
                    let y = data.next_f64_offset(last_point.1);

                    points.push((x as f32, y as f32));

                    last_point = (x, y);
                }

                Some(ElementEdit::SetControlPoints(points))
            }

            'P' => {
                let num_components  = data.next_usize();
                let mut last_point  = PathPoint::new(0.0, 0.0);
                let mut components  = vec![];

                for _ in 0..num_components {
                    if let Some((component, next_point)) = PathComponent::deserialize_next(&last_point, data) {
                        components.push(component);
                        last_point = next_point;
                    } else {
                        return None;
                    }
                }

                Some(ElementEdit::SetPath(Arc::new(components)))
            }

            _   => None
        }
    }
}
