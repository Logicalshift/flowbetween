use crate::editor::*;
use crate::storage::*;
use crate::traits::*;

use flo_stream::*;
use futures::prelude::*;
use futures::executor;

use std::sync::*;

#[test]
fn retrieve_default_size() {
    // Create an animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Default animation size is 1920x1080
    let size            = animation.size();
    assert!(size == (1920.0, 1080.0));
}

#[test]
fn set_and_retrieve_size() {
    // Create an animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    executor::block_on(async {
        animation.edit().publish(Arc::new(vec![AnimationEdit::SetSize(1080.0, 720.0)])).await;
    });

    // Default animation size is 1920x1080
    let size            = animation.size();
    assert!(size == (1080.0, 720.0));
}

