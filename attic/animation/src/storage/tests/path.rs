use super::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn read_path_element() {
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
            ]))))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let element100          = frame.element_with_id(ElementId::Assigned(100)).unwrap();

    if let Vector::Path(_path) = element100 {
        assert!(true);
    } else {
        // Not a path
        assert!(false);
    }
}

#[test]
fn update_path_elements() {
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
            ]))))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::SetControlPoints(vec![
            (60.0, 60.0),
            (70.0, 70.0),
            (80.0, 90.0), (100.0, 110.0), (120.0, 130.0)
        ], Duration::from_millis(300)))
    ]);


    let edit_log        = anim.read_edit_log(5..6);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    if let AnimationEdit::Element(ref elem_ids, ElementEdit::SetControlPoints(ref control_points, ref _when)) = edits[0] {
        assert!(elem_ids == &vec![ElementId::Assigned(100)]);
        assert!(control_points == &vec![
            (60.0, 60.0),
            (70.0, 70.0),
            (80.0, 90.0), (100.0, 110.0), (120.0, 130.0)
        ]);
    } else {
        assert!(false);
    }
}

#[test]
fn replace_path_components() {
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
            ]))))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::SetPath(Arc::new(vec![
            PathComponent::Move(PathPoint::new(50.0, 100.0)),
            PathComponent::Line(PathPoint::new(60.0, 110.0)),
            PathComponent::Line(PathPoint::new(70.0, 120.0)),
            PathComponent::Close
        ])))
    ]);


    let edit_log        = anim.read_edit_log(5..6);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(edits.len() > 0);

    if let AnimationEdit::Element(ref elem_ids, ElementEdit::SetPath(ref new_path)) = edits[0] {
        assert!(elem_ids == &vec![ElementId::Assigned(100)]);
        assert!(new_path == &Arc::new(vec![
            PathComponent::Move(PathPoint::new(50.0, 100.0)),
            PathComponent::Line(PathPoint::new(60.0, 110.0)),
            PathComponent::Line(PathPoint::new(70.0, 120.0)),
            PathComponent::Close
        ]));
    } else {
        assert!(false);
    }
}

#[test]
fn create_path_and_re_order_behind() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(100))),
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
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(102), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(101)], ElementEdit::Order(ElementOrdering::Behind))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(101));
        assert!(elements[1].id() == ElementId::Assigned(100));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(101)], ElementEdit::Order(ElementOrdering::InFront))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    let edit_log        = anim.read_edit_log(7..8);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    if let AnimationEdit::Element(ref elems, ElementEdit::Order(ElementOrdering::Behind)) = edits[0] {
        assert!(elems == &vec![ElementId::Assigned(101)]);
    } else {
        assert!(false);
    }
}

#[test]
fn create_path_and_re_order_in_front() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(100))),
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
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(102), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Order(ElementOrdering::InFront))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(101));
        assert!(elements[1].id() == ElementId::Assigned(100));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Order(ElementOrdering::Behind))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    let edit_log        = anim.read_edit_log(7..8);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    if let AnimationEdit::Element(ref elems, ElementEdit::Order(ElementOrdering::InFront)) = edits[0] {
        assert!(elems == &vec![ElementId::Assigned(100)]);
    } else {
        assert!(false);
    }
}

#[test]
fn create_path_and_re_order_before() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(100))),
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
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(102), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Order(ElementOrdering::Before(ElementId::Assigned(102))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(101));
        assert!(elements[1].id() == ElementId::Assigned(100));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }
}

#[test]
fn create_path_and_re_order_to_top_and_bottom() {
    use self::LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(100))),
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
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(102), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Order(ElementOrdering::ToTop))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(101));
        assert!(elements[1].id() == ElementId::Assigned(102));
        assert!(elements[2].id() == ElementId::Assigned(100));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Order(ElementOrdering::ToBottom))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() != 1);
        assert!(elements.len() <= 3);
        assert!(elements.len() == 3);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
        assert!(elements[2].id() == ElementId::Assigned(102));
    }
}
