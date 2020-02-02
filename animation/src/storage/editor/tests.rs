use super::*;
use super::super::*;
use super::super::super::traits::*;

use futures::prelude::*;

#[test]
fn retrieve_default_size() {
    // Create an animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Default animation size is 1920x1080
    let size            = animation.size();
    assert!(size == (1920.0, 1080.0));
}

