use crate::editor::*;
use crate::storage::*;
use crate::traits::*;

use flo_stream::*;
use futures::prelude::*;
use futures::executor;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::sync::*;
use std::time::{Duration};

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

    // Update the size
    executor::block_on(async {
        animation.edit().publish(Arc::new(vec![AnimationEdit::SetSize(1080.0, 720.0)])).await;
    });

    // Default animation size is 1920x1080 so it should now be updated
    let size            = animation.size();
    assert!(size == (1080.0, 720.0));
}

#[test]
fn send_retired_instructions() {
    // Create an animation
    let in_memory_store     = InMemoryStorage::new();
    let animation           = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
    let mut edits           = animation.edit();
    let mut retired_edits   = animation.retired_edits();

    // Update the size
    executor::block_on(async {
        edits.publish(Arc::new(vec![AnimationEdit::SetSize(1080.0, 720.0)])).await;
        edits.when_empty().await;
    });

    // Should get a message back when it retires
    let edits               = executor::block_on(async {
        let retirement_timeout  = Delay::new(Duration::from_secs(10));
        select(retired_edits.next(), retirement_timeout).await
    });

    let edits               = match edits {
        Either::Right(_)    => { assert!(false, "Timed out"); unimplemented!() }
        Either::Left(edits) => edits.0.unwrap().committed_edits()
    };

    assert!(edits == Arc::new(vec![AnimationEdit::SetSize(1080.0, 720.0)]));
}
