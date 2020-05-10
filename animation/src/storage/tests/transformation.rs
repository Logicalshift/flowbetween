use super::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn translate_path() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Transform(vec![ElementTransform::SetAnchor(10.0, 20.0), ElementTransform::MoveTo(0.0, 0.0)]))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let _element100         = frame.element_with_id(ElementId::Assigned(100)).unwrap();
    let attachments         = frame.attached_elements(ElementId::Assigned(100));

    // Should be a transformation attached
    assert!(attachments.len() == 1);
    let attached_element    =  frame.element_with_id(attachments[0].0).unwrap();

    assert!(if let Vector::Transformation((_, _)) = attached_element { true } else { false });

    if let Vector::Transformation((_, transform)) = attached_element {
        assert!(transform.len() == 1);

        if let Transformation::Translate(x, y) = transform[0] {
            assert!((x- -10.0).abs() < 0.001);
            assert!((y- -20.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

#[test]
fn align_paths_left() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, BrushProperties::new()))),

        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(101), Arc::new(vec![
                PathComponent::Move(PathPoint::new(5.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),

        AnimationEdit::Element(vec![ElementId::Assigned(100), ElementId::Assigned(101)], ElementEdit::Transform(vec![ElementTransform::Align(ElementAlign::Left)]))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let _element100         = frame.element_with_id(ElementId::Assigned(100)).unwrap();
    let attachments         = frame.attached_elements(ElementId::Assigned(100));

    // Should be a transformation attached to element ID 100
    assert!(attachments.len() == 1);
    let attached_element    =  frame.element_with_id(attachments[0].0).unwrap();

    assert!(if let Vector::Transformation((_, _)) = attached_element { true } else { false });

    if let Vector::Transformation((_, transform)) = attached_element {
        assert!(transform.len() == 1);

        if let Transformation::Translate(x, y) = transform[0] {
            assert!((x- -5.0).abs() < 0.001);
            assert!((y- 0.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}


#[test]
fn align_paths_top() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, BrushProperties::new()))),

        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(101), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 45.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),

        AnimationEdit::Element(vec![ElementId::Assigned(100), ElementId::Assigned(101)], ElementEdit::Transform(vec![ElementTransform::Align(ElementAlign::Top)]))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let _element100         = frame.element_with_id(ElementId::Assigned(100)).unwrap();
    let attachments         = frame.attached_elements(ElementId::Assigned(100));

    // Should be a transformation attached to element ID 100
    assert!(attachments.len() == 1);
    let attached_element    =  frame.element_with_id(attachments[0].0).unwrap();

    assert!(if let Vector::Transformation((_, _)) = attached_element { true } else { false });

    if let Vector::Transformation((_, transform)) = attached_element {
        assert!(transform.len() == 1);

        if let Transformation::Translate(x, y) = transform[0] {
            assert!((x- 0.0).abs() < 0.001);
            assert!((y- 5.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

#[test]
fn align_paths_center() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, BrushProperties::new()))),

        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(101), Arc::new(vec![
                PathComponent::Move(PathPoint::new(5.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),

        AnimationEdit::Element(vec![ElementId::Assigned(100), ElementId::Assigned(101)], ElementEdit::Transform(vec![ElementTransform::Align(ElementAlign::Center)]))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let _element100         = frame.element_with_id(ElementId::Assigned(100)).unwrap();
    let attachments         = frame.attached_elements(ElementId::Assigned(100));

    // Should be a transformation attached to element ID 100
    assert!(attachments.len() == 1);
    let attached_element    =  frame.element_with_id(attachments[0].0).unwrap();

    assert!(if let Vector::Transformation((_, _)) = attached_element { true } else { false });

    if let Vector::Transformation((_, transform)) = attached_element {
        assert!(transform.len() == 1);

        if let Transformation::Translate(x, y) = transform[0] {
            assert!((x- -2.5).abs() < 0.001);
            assert!((y- 0.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}
