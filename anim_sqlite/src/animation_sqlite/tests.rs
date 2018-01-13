use super::*;
use animation::*;

#[test]
fn default_size_is_1980_1080() {
    let anim = SqliteAnimation::new_in_memory();

    assert!(anim.size() == (1980.0, 1080.0));
}

#[test]
fn no_layers_by_default() {
    let anim = SqliteAnimation::new_in_memory();

    assert!(anim.get_layer_ids().len() == 0);
}
