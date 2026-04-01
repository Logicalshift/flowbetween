use super::*;

use std::time::Duration;

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
