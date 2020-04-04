use super::editor::*;
use crate::*;
use crate::storage::*;
use futures::*;

mod animation_properties;
mod layers;
mod edit_log;
mod frame_edits;
mod motion;
mod path;
mod caching;

///
/// Creates an in-memory animaton for the tests
///
pub fn create_animation() -> impl EditableAnimation {
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    animation
}
