use super::*;
use canvas::*;
use animation::*;
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
        let mut layer_editor = anim.edit_layer(2);
        layer_editor.set_pending(&[LayerEdit::AddKeyFrame(Duration::from_millis(250))]);
        layer_editor.commit_pending();
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

    let edit_log    = anim.get_log();
    let edits       = edit_log.read(&mut (0..7));

    assert!(edits.len() == 7);
    assert!(edits[0] == AnimationEdit::AddNewLayer(2));
    assert!(edits[1] == AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))));
    assert!(edits[2] == AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )));
    assert!(edits[3] == AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))));
    assert!(edits[6] == AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))));
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
    let edit_log = animation.get_log();

    let paint_edit = edit_log.read(&mut (4..5));

    // Should be able to find the paint edit here
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, _)) => true, _ => false });

    // Element ID should be assigned
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, PaintEdit::BrushStroke(ElementId::Assigned(_), _))) => true, _ => false });
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

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(442));
        assert!(elements.len() == 5);

        assert!(match &elements[0] {
            &Vector::BrushDefinition(ref defn) => Some(defn.definition()),
            _ => None
        } == Some(&BrushDefinition::Ink(InkDefinition::default())));

        assert!(match &elements[1] {
            &Vector::BrushProperties(ref props) => Some(props.brush_properties()),
            _ => None
        } == Some(&BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }));


        assert!(match &elements[3] {
            &Vector::BrushStroke(ref brush_stroke) => Some(brush_stroke.points()),
            _ => None
        }.is_some());
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
