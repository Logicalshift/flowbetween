use crate::*;
use crate::editor::*;
use crate::storage::*;
use futures::*;

mod animation_properties;
mod layers;
mod edit_log;
mod frame_edits;
mod motion;
mod path;
mod caching;
mod collide_paths;
mod grouping;
mod transformation;

///
/// Creates an in-memory animaton for the tests
///
pub fn create_animation() -> impl EditableAnimation {
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    animation
}

///
/// Deserializes some edits and runs them on the animation. The edit string can be generated
/// by the diagnostics command line tool.
///
pub fn perform_serialized_edits<Anim: EditableAnimation>(animation: &mut Anim, edits: &str) {
    let mut source              = edits.chars();
    let mut deserialized_edits  = vec![];

    // Deserialize the edits
    loop {
        match source.next() {
            None        => { break; },
            Some('\n')  |
            Some(' ')   |
            Some('\t')  |
            Some('\r')  => { /* Newlines and whitespace are ignored */ },

            Some(';')   => {
                // Comment ended by a newline
                loop {
                    match source.next() {
                        Some('\n')  |
                        Some('\r')  | 
                        None        => { break; },
                        _other      => { }
                    }
                }
            }

            Some(other) => {
                // Serialized edit, ended by a newline
                let mut edit = String::new();
                edit.push(other);

                // Read until the newline to get the serialized edit
                loop {
                    match source.next() {
                        Some('\n')  => { break; }
                        None        => { break; },
                        Some(other) => { edit.push(other); }
                    }
                }

                // Attempt to deserialize the edit
                let animation_edit = AnimationEdit::deserialize(&mut edit.chars());

                // Add to the edit buffer
                animation_edit.map(|animation_edit| deserialized_edits.push(animation_edit));
            }
        }
    }

    // Send to the animation
    animation.perform_edits(deserialized_edits);
}
