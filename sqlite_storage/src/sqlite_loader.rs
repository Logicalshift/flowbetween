use super::sqlite_storage::*;

use flo_animation::*;
use flo_animation::storage::*;

use futures::prelude::*;
use rusqlite::{Connection, OpenFlags};

///
/// Creates a loader for loading animations stored in SQLite files
///
pub fn sqlite_animation_loader() -> impl FileAnimation {
    AnimationLoader(|path| {
        // Connect to the database
        let opening_existing = path.exists();

        let storage = if opening_existing {
            // Open/restore an existing animation
            let connection  = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
            let storage     = SqliteAnimationStorage::from_connection(connection);

            storage
        } else {
            // Create a new animation
            let connection  = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE).unwrap();
            let storage     = SqliteAnimationStorage::new_from_connection(connection);

            storage
        };

        // Create the editor for this animation
        let editor      = create_animation_editor(move |commands| storage.get_responses(commands).boxed());

        if !opening_existing {
            // Set up a default animation
            editor.perform_edits(vec![
                AnimationEdit::SetSize(1920.0, 1080.0),
                AnimationEdit::AddNewLayer(0),
                AnimationEdit::Layer(0, LayerEdit::SetName("Layer 1".to_string()))
            ]);
        }

        editor
    })
}
