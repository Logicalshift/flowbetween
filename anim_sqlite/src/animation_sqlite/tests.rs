use super::*;
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

    let keyframes: Vec<Duration> = layer.unwrap().get_key_frames().collect();
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

    let keyframes: Vec<Duration> = layer.unwrap().get_key_frames().collect();
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

    let keyframes: Vec<Duration> = layer.unwrap().get_key_frames().collect();
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

/*
#[test]
fn draw_brush_strokes() {
    let anim = SqliteAnimation::new_in_memory();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);
    anim.panic_on_error();
}
*/