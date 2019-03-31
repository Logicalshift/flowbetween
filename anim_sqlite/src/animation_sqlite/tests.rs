use super::*;

use flo_canvas::*;
use flo_animation::*;

use futures::*;
use futures::executor;
use std::time::Duration;
use std::sync::*;

#[test]
fn default_size_is_1980_1080() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    assert!(anim.size() == (1980.0, 1080.0));
}

#[test]
fn default_duration_is_two_minutes() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    assert!(anim.duration() == Duration::from_secs(120));
}

#[test]
fn default_framerate_is_30_fps() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    assert!(anim.frame_length() == Duration::new(0, 33_333_333));
}

#[test]
fn no_layers_by_default() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    assert!(anim.get_layer_ids().len() == 0);
}

#[test]
fn set_size() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::SetSize(100.0, 200.0)
    ]);

    anim.panic_on_error();
}

#[test]
fn size_changes_after_being_set() {
    let anim = SqliteAnimation::new_in_memory();

    assert!(anim.size() == (1980.0, 1080.0));

    anim.perform_edits(vec![
        AnimationEdit::SetSize(100.0, 200.0)
    ]);

    assert!((anim.size().0-100.0).abs() < 0.01);
    assert!((anim.size().1-200.0).abs() < 0.01);

    anim.panic_on_error();
}

#[test]
fn add_layer() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 1);

    anim.panic_on_error();
}

#[test]
fn add_multiple_layers() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(3),
        AnimationEdit::AddNewLayer(4)
    ]);
    anim.panic_on_error();

    assert!(anim.get_layer_ids().len() == 3);

    anim.panic_on_error();
}

#[test]
fn remove_layer() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);
    anim.panic_on_error();

    assert!(anim.get_layer_ids().len() == 1);

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);
    anim.panic_on_error();

    assert!(anim.get_layer_ids().len() == 0);

    anim.panic_on_error();
}

#[test]
fn cannot_add_same_layer_again() {
    let anim = SqliteAnimation::new_in_memory();
    anim.panic_on_error();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.retrieve_and_clear_error().is_none());

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.retrieve_and_clear_error().is_some());
}

#[test]
fn single_layer_id() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 1);
    assert!(layer_ids == vec![2]);

    anim.panic_on_error();
}

#[test]
fn retrieve_layer_ids() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(42)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 2);
    assert!(layer_ids == vec![2, 42]);

    anim.panic_on_error();
}

#[test]
fn retrieve_layer() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());
    assert!(layer.unwrap().id() == 2);

    anim.panic_on_error();
}

#[test]
fn non_existent_layer() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(3);
    assert!(layer.is_none());

    anim.panic_on_error();
}

#[test]
fn add_keyframe() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);

    anim.panic_on_error();

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe2() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);

    anim.panic_on_error();

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe_with_layer_editor() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    {
        let mut sink = executor::spawn(anim.edit());

        sink.wait_send(vec![AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))]).unwrap();
        sink.wait_flush().unwrap();
    }

    anim.panic_on_error();

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn remove_keyframe() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::RemoveKeyFrame(Duration::from_millis(250)))
    ]);

    anim.panic_on_error();
}

#[test]
fn retrieve_keyframe() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);

    anim.panic_on_error();
}

#[test]
fn find_previus_and_next_keyframe() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(500))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(750)))
    ]);

    anim.panic_on_error();

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
    let anim = SqliteAnimation::new_in_memory();

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

    anim.panic_on_error();
}

#[test]
fn draw_brush_strokes() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.panic_on_error();
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
    anim.panic_on_error();
}

#[test]
fn draw_brush_strokes_in_future() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.panic_on_error();
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
    anim.panic_on_error();
}

#[test]
fn edit_brush_strokes() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.panic_on_error();
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(100), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        
        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::SetControlPoints(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)]))
    ]);
    anim.panic_on_error();
}

#[test]
fn read_brush_strokes_from_edit_log() {
    let anim = SqliteAnimation::new_in_memory();

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
    anim.panic_on_error();

    let edit_log        = anim.read_edit_log(0..7);
    let edit_log        = edit_log.collect();
    let mut edit_log    = executor::spawn(edit_log);
    let edits           = edit_log.wait_future().unwrap();

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
fn will_assign_element_ids() {
    let animation = SqliteAnimation::new_in_memory();;

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
    let edit_log        = animation.read_edit_log(4..5);
    let edit_log        = edit_log.collect();
    let mut edit_log    = executor::spawn(edit_log);

    let paint_edit = edit_log.wait_future().unwrap();

    // Should be able to find the paint edit here
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, _)) => true, _ => false });

    // Element ID should be assigned
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, PaintEdit::BrushStroke(ElementId::Assigned(_), _))) => true, _ => false });
}

#[test]
fn fetch_element_by_id() {
    let anim = SqliteAnimation::new_in_memory();

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
    anim.panic_on_error();

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
    let anim = SqliteAnimation::new_in_memory();

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
    anim.panic_on_error();

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
    }

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(60));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(60));
        assert!(elements.len() == 0);
    }
}

#[test]
fn delete_layer_after_drawing_brush_stroke() {
    let anim = SqliteAnimation::new_in_memory();

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
    anim.panic_on_error();

    anim.perform_edits(vec![AnimationEdit::RemoveLayer(2)]);
}

#[test]
fn move_existing_element() {
    let anim = SqliteAnimation::new_in_memory();

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
        AnimationEdit::Motion(ElementId::Assigned(100), MotionEdit::Attach(ElementId::Assigned(50)))
    ]);
    anim.panic_on_error();

    let attached = anim.get_motions_for_element(ElementId::Assigned(50));
    assert!(attached == vec![ElementId::Assigned(100)]);

    let motion = anim.get_motion(ElementId::Assigned(100));
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
fn read_path_element() {
    use self::LayerEdit::*;

    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, flo_animation::BrushProperties::new()))),
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
fn create_path_and_re_order() {
    use self::LayerEdit::*;

    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(100))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, flo_animation::BrushProperties::new()))),
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
            ]))))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 2);
        assert!(elements[0].id() == ElementId::Assigned(100));
        assert!(elements[1].id() == ElementId::Assigned(101));
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(101)], ElementEdit::Order(ElementOrdering::Behind))
    ]);

    {
        let layer               = anim.get_layer_with_id(24).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(300));
        let elements            = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 2);
        assert!(elements[0].id() == ElementId::Assigned(101));
        assert!(elements[1].id() == ElementId::Assigned(100));
    }
}
