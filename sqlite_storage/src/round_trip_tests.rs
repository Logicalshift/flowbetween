use super::*;

use flo_animation::*;
use flo_animation::storage::*;
use flo_stream::*;
use flo_canvas::*;

use futures::*;
use futures::executor;
use std::time::Duration;
use std::sync::*;

///
/// Creates an in-memory animaton for the tests
///
fn create_animation() -> impl EditableAnimation {
    let sqlite_store    = SqliteAnimationStorage::new_from_connection(rusqlite::Connection::open_in_memory().unwrap());
    let animation       = create_animation_editor(move |commands| sqlite_store.get_responses(commands).boxed());

    animation
}

#[test]
fn default_size_is_1920_1080() {
    let anim = create_animation();

    assert!(anim.size() == (1920.0, 1080.0));
}

#[test]
fn default_duration_is_two_minutes() {
    let anim = create_animation();

    assert!(anim.duration() == Duration::from_secs(120));
}

#[test]
fn default_framerate_is_30_fps() {
    let anim = create_animation();

    assert!(anim.frame_length() == Duration::new(0, 33_333_333));
}

#[test]
fn no_layers_by_default() {
    let anim = create_animation();

    assert!(anim.get_layer_ids().len() == 0);
}

#[test]
fn set_size() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::SetSize(100.0, 200.0)
    ]);

}

#[test]
fn size_changes_after_being_set() {
    let anim = create_animation();

    assert!(anim.size() == (1920.0, 1080.0));

    anim.perform_edits(vec![
        AnimationEdit::SetSize(100.0, 200.0)
    ]);

    assert!((anim.size().0-100.0).abs() < 0.01);
    assert!((anim.size().1-200.0).abs() < 0.01);

}

#[test]
fn add_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 1);

}

#[test]
fn add_multiple_layers() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(3),
        AnimationEdit::AddNewLayer(4)
    ]);

    assert!(anim.get_layer_ids().len() == 3);

}

#[test]
fn remove_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 1);

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 0);

}

#[test]
fn cannot_add_same_layer_again() {
    /*
    assert!(false); // TODO

    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.retrieve_and_clear_error().is_none());

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.retrieve_and_clear_error().is_some());
    */
}

#[test]
fn single_layer_id() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 1);
    assert!(layer_ids == vec![2]);

}

#[test]
fn retrieve_layer_ids() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(42)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 2);
    assert!(layer_ids == vec![2, 42]);

}

#[test]
fn retrieve_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());
    assert!(layer.unwrap().id() == 2);

}

#[test]
fn non_existent_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(3);
    assert!(layer.is_none());

}

#[test]
fn add_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe2() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe_with_layer_editor() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    executor::block_on(async {
        let mut sink = anim.edit();

        sink.publish(Arc::new(vec![AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))])).await;
        sink.when_empty().await;
    });


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn remove_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::RemoveKeyFrame(Duration::from_millis(250)))
    ]);

}

#[test]
fn retrieve_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);

}

#[test]
fn find_previous_and_next_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(500))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(750)))
    ]);


    let layer = anim.get_layer_with_id(2).unwrap();

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(375));
    assert!(previous == Some(Duration::from_millis(250)));
    assert!(next == Some(Duration::from_millis(500)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(625));
    assert!(previous == Some(Duration::from_millis(500)));
    assert!(next == Some(Duration::from_millis(750)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(1000));
    assert!(previous == Some(Duration::from_millis(750)));
    assert!(next == None);

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(0));
    assert!(previous == None);
    assert!(next == Some(Duration::from_millis(250)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(500));
    assert!(previous == Some(Duration::from_millis(250)));
    assert!(next == Some(Duration::from_millis(750)));
}

#[test]
fn remove_layer_with_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_none());

}

#[test]
fn draw_brush_strokes() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);
}

#[test]
fn draw_brush_strokes_in_future() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(450), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(500), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);
}

#[test]
fn edit_brush_strokes() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(100), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::SetControlPoints(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], Duration::from_millis(442)))
    ]);
}

#[test]
fn read_brush_strokes_from_edit_log() {
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
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    let edit_log        = anim.read_edit_log(0..7);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(edits.len() == 7);
    assert!(edits[0] == AnimationEdit::AddNewLayer(2));
    assert!(edits[1] == AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))));
    assert!(match edits[2] {
        AnimationEdit::Layer(2, LayerEdit::Paint(_when, PaintEdit::SelectBrush(
                ElementId::Assigned(_element_id),
                BrushDefinition::Ink(ref ink_defn),
                BrushDrawingStyle::Draw
            )
        ))  => ink_defn == &InkDefinition::default(),
        _   => false
    });
    assert!(match edits[3] {
        AnimationEdit::Layer(2, LayerEdit::Paint(_when, PaintEdit::
            BrushProperties(ElementId::Assigned(_element_id), ref brush_properties)))
                => brush_properties == &BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 },
            _ => false
    });
    assert!(match edits[6] {
        AnimationEdit::Layer(2, LayerEdit::Paint(ref when, PaintEdit::BrushStroke(ElementId::Assigned(_), ref points)))
                        => points == &Arc::new(vec![
                            RawPoint::from((10.0, 10.0)),
                            RawPoint::from((20.0, 5.0))
                        ]) && when == &Duration::from_millis(442),
                _       => false
    });
}

#[test]
fn read_element_delete_from_edit_log() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(50))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(126), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(127), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Element(vec![ElementId::Assigned(127), ElementId::Assigned(128), ElementId::Assigned(126)], ElementEdit::Delete)
    ]);

    let edit_log        = anim.read_edit_log(7..8);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(edits.len() == 1);
    assert!(edits[0] == AnimationEdit::Element(vec![ElementId::Assigned(127), ElementId::Assigned(128), ElementId::Assigned(126)], ElementEdit::Delete));
}

#[test]
fn will_assign_element_ids() {
    let animation = create_animation();

    // Perform some edits on the animation with an unassigned element ID
    animation.perform_edits(vec![
        AnimationEdit::AddNewLayer(0),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((20.0, 5.0))
            ]))))
    ]);

    // Element ID should be assigned if we read the log back
    let edit_log            = animation.read_edit_log(4..5);
    let edit_log            = edit_log.collect();

    let paint_edit: Vec<_>  = executor::block_on(edit_log);

    // Should be able to find the paint edit here
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, _)) => true, _ => false });

    // Element ID should be assigned
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, PaintEdit::BrushStroke(ElementId::Assigned(_), _))) => true, _ => false });
}

#[test]
fn fetch_element_by_id() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(50))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(126), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(127), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element127          = frame.element_with_id(ElementId::Assigned(127));

        assert!(match element127 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == Some(ElementId::Assigned(127)));
    }

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(60));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(60));
        assert!(elements.len() == 0);
    }
}

#[test]
fn read_frame_after_edits() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(50))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(126), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(127), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(442));
        assert!(elements.len() == 3);

        assert!(match &elements[1] {
            &Vector::BrushStroke(ref brush_stroke) => Some(brush_stroke.points()),
            _ => None
        }.is_some());

        assert!(match &elements[1] {
            &Vector::BrushStroke(ref brush_stroke) => Some(brush_stroke.id()),
            _ => None
        } == Some(ElementId::Assigned(127)));

        // All our elements should have the brush properties attached to them
        let attachments = frame.attached_elements(ElementId::Assigned(126));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);

        let attachments = frame.attached_elements(ElementId::Assigned(127));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);

        let attachments = frame.attached_elements(ElementId::Assigned(128));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);
    }

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(60));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(60));
        assert!(elements.len() == 0);
    }
}

#[test]
fn delete_element() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(50))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(126), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(127), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Element(vec![ElementId::Assigned(127)], ElementEdit::Delete)
    ]);

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element127          = frame.element_with_id(ElementId::Assigned(127));

        assert!(match element127 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == None);
    }
}

#[test]
fn delete_layer_after_drawing_brush_stroke() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    anim.perform_edits(vec![AnimationEdit::RemoveLayer(2)]);
}

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

/*
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
*/

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

#[test]
fn set_and_retrieve_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == Some(Arc::new(vec![Draw::NewPath, Draw::Fill])));
}

#[test]
fn retrieve_or_generate_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    let cached_drawing  = cache.retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(|| Arc::new(vec![Draw::NewPath, Draw::Fill])));

    // Should initially be a future indicating the cached item will be generated eventually
    assert!(match cached_drawing { CacheProcess::Process(_) => true, _ => false });

    // ... and eventually evaluate to the drawing we specified in the generate function
    let cached_drawing = executor::block_on(cached_drawing);

    assert!(cached_drawing == Arc::new(vec![Draw::NewPath, Draw::Fill]));

    // Should be able to retrieve instantly next time
    let cached_drawing  = cache.retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(|| Arc::new(vec![Draw::NewPath, Draw::Fill])));

    assert!(match cached_drawing { CacheProcess::Cached(_) => true, _ => false });
    assert!(match cached_drawing { CacheProcess::Cached(cached_drawing) => cached_drawing == Arc::new(vec![Draw::NewPath, Draw::Fill]), _ => false });
}

#[test]
fn invalidate_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));
    cache.invalidate(CacheType::OnionSkinLayer);

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == None);
}

#[test]
fn retrieve_cached_onionskin_from_different_time() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));
    cache.invalidate(CacheType::OnionSkinLayer);

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(1500));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == None);
}
