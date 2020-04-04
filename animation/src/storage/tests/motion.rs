use super::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn read_motion_edit_items() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let edit_log        = anim.read_edit_log(5..10);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(match edits[0] {
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create) => true,
        _ => false
    });
    assert!(match edits[1] {
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)) => true,
        _ => false
    });
    assert!(match edits[2] {
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(x, y)) => (x-50.0).abs() < 0.01 && (y-60.0).abs() < 0.01,
        _ => false
    });

    if let AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(ref curve)) = edits[3] {
        assert!(curve == &TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))));
    } else {
        assert!(false);
    }

    if let AnimationEdit::Element(ref elem_ids, ElementEdit::AddAttachment(ElementId::Assigned(100))) = edits[4] {
        assert!(elem_ids == &vec![ElementId::Assigned(50)]);
    } else {
        assert!(false);
    }
}

#[test]
fn move_existing_element() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![ElementId::Assigned(100)]);

    let motion = anim.motion().get_motion(ElementId::Assigned(100));
    assert!(motion.is_some());
    assert!(motion.as_ref().unwrap().motion_type() == MotionType::Translate);

    if let Some(Motion::Translate(translate)) = motion {
        assert!(translate.origin == (50.0, 60.0));
        assert!(translate.translate == TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))));
    } else {
        assert!(false)
    }
}

#[test]
fn read_elements_for_motion() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![ElementId::Assigned(50)]);
}

#[test]
fn detach_motion() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![ElementId::Assigned(50)]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![ElementId::Assigned(100)]);

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::RemoveAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![]);
}

#[test]
fn delete_motion() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![ElementId::Assigned(50)]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![ElementId::Assigned(100)]);

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::Delete)
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![]);
}

#[test]
fn delete_motion_element() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(50), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetOrigin(50.0, 60.0)),
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::SetPath(TimeCurve::new(TimePoint::new(200.0, 200.0, Duration::from_millis(442)), TimePoint::new(200.0, 200.0, Duration::from_millis(442))))),
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::AddAttachment(ElementId::Assigned(100)))
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![ElementId::Assigned(50)]);

    let attached = anim.motion().get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![ElementId::Assigned(100)]);

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(50)], ElementEdit::Delete)
    ]);

    let attached = anim.motion().get_elements_for_motion(ElementId::Assigned(100));
    assert!(attached == vec![]);
}
