use super::super::source::*;
use super::super::target::*;

use crate::traits::*;

use std::sync::*;
use std::time::{Duration};

impl ElementEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementEdit::*;

        match self {
            AddAttachment(elem)             => { data.write_chr('+'); elem.serialize(data); }
            RemoveAttachment(elem)          => { data.write_chr('-'); elem.serialize(data); }
            Order(ordering)                 => { data.write_chr('O'); ordering.serialize(data); }
            Delete                          => { data.write_chr('X'); }
            DetachFromFrame                 => { data.write_chr('D'); }
            CollideWithExistingElements     => { data.write_chr('j'); }
            ConvertToPath                   => { data.write_chr('p'); }
            Group(group_id, group_type)     => { data.write_chr('g'); group_id.serialize(data); group_type.serialize(data); }
            Ungroup                         => { data.write_chr('u'); }

            SetControlPoints(points, when)  => { 
                data.write_chr('c');
                data.write_duration(*when); 
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
            },

            Transform(transform)        => {
                data.write_chr('T');
                data.write_usize(transform.len());

                transform.iter().for_each(|transform| transform.serialize(data));
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

            'j' => {
                Some(ElementEdit::CollideWithExistingElements)
            }

            'p' => {
                Some(ElementEdit::ConvertToPath)
            }

            'g' => {
                Some(ElementEdit::Group(ElementId::deserialize(data)?, GroupType::deserialize(data)?))
            }

            'u' => {
                Some(ElementEdit::Ungroup)
            }

            'C' => {
                // Obsolete version from older versions of FlowBetween
                let num_points      = data.next_usize();
                let mut last_point  = (0.0, 0.0);
                let mut points      = vec![];

                for _ in 0..num_points {
                    let x = data.next_f64_offset(last_point.0);
                    let y = data.next_f64_offset(last_point.1);

                    points.push((x as f32, y as f32));

                    last_point = (x, y);
                }

                Some(ElementEdit::SetControlPoints(points, Duration::from_millis(0)))
            }

            'c' => {
                let when            = data.next_duration();
                let num_points      = data.next_usize();
                let mut last_point  = (0.0, 0.0);
                let mut points      = vec![];

                for _ in 0..num_points {
                    let x = data.next_f64_offset(last_point.0);
                    let y = data.next_f64_offset(last_point.1);

                    points.push((x as f32, y as f32));

                    last_point = (x, y);
                }

                Some(ElementEdit::SetControlPoints(points, when))
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

            'T' => {
                let num_transforms = data.next_usize();
                let mut transforms = vec![];

                for _ in 0..num_transforms {
                    if let Some(transform) = ElementTransform::deserialize(data) {
                        transforms.push(transform)
                    } else {
                        return None;
                    }
                }

                Some(ElementEdit::Transform(transforms))
            }

            _   => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_attachment() {
        let mut encoded = String::new();
        ElementEdit::AddAttachment(ElementId::Assigned(42)).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::AddAttachment(ElementId::Assigned(42))));
    }

    #[test]
    fn remove_attachment() {
        let mut encoded = String::new();
        ElementEdit::RemoveAttachment(ElementId::Assigned(42)).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::RemoveAttachment(ElementId::Assigned(42))));
    }

    #[test]
    fn ordering() {
        let mut encoded = String::new();
        ElementEdit::Order(ElementOrdering::InFront).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Order(ElementOrdering::InFront)));
    }

    #[test]
    fn delete() {
        let mut encoded = String::new();
        ElementEdit::Delete.serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Delete));
    }

    #[test]
    fn detach_from_frame() {
        let mut encoded = String::new();
        ElementEdit::DetachFromFrame.serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::DetachFromFrame));
    }

    #[test]
    fn collide_with_existing() {
        let mut encoded = String::new();
        ElementEdit::CollideWithExistingElements.serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::CollideWithExistingElements));
    }

    #[test]
    fn convert_to_path() {
        let mut encoded = String::new();
        ElementEdit::ConvertToPath.serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::ConvertToPath));
    }

    #[test]
    fn set_control_points() {
        let mut encoded = String::new();
        ElementEdit::SetControlPoints(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)], Duration::from_millis(442)).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::SetControlPoints(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)], Duration::from_millis(442))));
    }

    #[test]
    fn set_path() {
        let mut encoded = String::new();
        ElementEdit::SetPath(Arc::new(vec![PathComponent::Move(PathPoint::new(1.0, 2.0)), PathComponent::Line(PathPoint::new(2.0, 3.0)), PathComponent::Bezier(PathPoint::new(4.0, 5.0), PathPoint::new(6.0, 7.0), PathPoint::new(8.0, 9.0)), PathComponent::Close])).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::SetPath(Arc::new(vec![PathComponent::Move(PathPoint::new(1.0, 2.0)), PathComponent::Line(PathPoint::new(2.0, 3.0)), PathComponent::Bezier(PathPoint::new(4.0, 5.0), PathPoint::new(6.0, 7.0), PathPoint::new(8.0, 9.0)), PathComponent::Close]))));
    }

    #[test]
    fn group() {
        let mut encoded = String::new();
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal)));
    }

    #[test]
    fn group_added() {
        let mut encoded = String::new();
        ElementEdit::Group(ElementId::Unassigned, GroupType::Added).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Group(ElementId::Unassigned, GroupType::Added)));
    }

    #[test]
    fn ungroup() {
        let mut encoded = String::new();
        ElementEdit::Ungroup.serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Ungroup));
    }

    #[test]
    fn transform_set_anchor() {
        let mut encoded = String::new();
        ElementEdit::Transform(vec![ElementTransform::SetAnchor(2.0, 3.0)]).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Transform(vec![ElementTransform::SetAnchor(2.0, 3.0)])));
    }

    #[test]
    fn transform_move() {
        let mut encoded = String::new();
        ElementEdit::Transform(vec![ElementTransform::MoveTo(2.0, 3.0)]).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Transform(vec![ElementTransform::MoveTo(2.0, 3.0)])));
    }

    #[test]
    fn transform_anchor_and_move() {
        let mut encoded = String::new();
        ElementEdit::Transform(vec![ElementTransform::SetAnchor(6.0, 7.0), ElementTransform::MoveTo(2.0, 3.0)]).serialize(&mut encoded);

        assert!(ElementEdit::deserialize(&mut encoded.chars()) == Some(ElementEdit::Transform(vec![ElementTransform::SetAnchor(6.0, 7.0), ElementTransform::MoveTo(2.0, 3.0)])));
    }
}
