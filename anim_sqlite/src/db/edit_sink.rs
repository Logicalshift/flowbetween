use super::*;
use super::animation_core::*;

use desync::{Desync, pipe_in};

///
/// Creates a publisher that writes edits to the a database
///
pub (crate) fn create_edit_publisher<TFile: 'static+FloFile+Unpin+Send>(db: &Arc<Desync<AnimationDbCore<TFile>>>) -> Publisher<Arc<Vec<AnimationEdit>>> {
    // Create our own copy of the database
    let db = Arc::clone(db);

    // Create a publisher
    let mut publisher = Publisher::new(1);

    // Pipe edits from a subscriber into the core
    pipe_in(db, publisher.subscribe(), |db, edits| process_edits(db, edits));

    // Return the publisher
    publisher
}

///
/// Processes a series of edits on a database core
///
fn process_edits<TFile: FloFile+Unpin+Send>(db: &mut AnimationDbCore<TFile>, edits: Arc<Vec<AnimationEdit>>) {
    // Clone the edits so we can modify them
    let edits = Vec::clone(&*edits);

    // Apply element IDs to the edits
    let edits = db.assign_element_ids(edits);

    // Add to the edit log
    db.failure = db.failure.take().or_else(|| db.insert_edits(&edits).err());

    // Perform the edits to the underlying data as well (provided the database error is clear)
    if db.failure.is_none() {
        // Queue the edits for a single transaction
        db.db.begin_queuing();

        // Perform the edits
        for edit in edits {
            db.failure = db.failure.take().or_else(|| db.perform_edit(edit).err());

            if let Some(ref failure) = db.failure {
                db.log.log((Level::Error, format!("Could not write edit log item: `{:?}`", failure)));
            }
        }

        // Update the database and set the final error, if there was one
        let execute_result  = db.db.execute_queue();

        if let Err(ref failure) = execute_result {
            db.log.log((Level::Error, format!("Could not complete editing operation: `{:?}`", failure)));
        }

        db.failure          = db.failure.take().or_else(move || execute_result.err());
    } else {
        db.log.log((Level::Error, format!("Cannot commit edits to animation due to earlier error: `{:?}`", db.failure)));
    }
}
