use super::*;

use animation::*;
use animation::editor::*;
use animation::inmemory::pending_log::*;

use rusqlite::*;
use std::time::Duration;

impl SqliteAnimation {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    pub fn retrieve_and_clear_error(&self) -> Option<Error> {
        self.db.retrieve_and_clear_error()
    }

    ///
    /// Panics if this animation has reached an error condition
    ///
    pub fn panic_on_error(&self) {
        self.retrieve_and_clear_error().map(|erm| panic!("{:?}", erm));
    }

    ///
    /// Convenience method that performs some edits on this animation
    /// 
    pub fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        let mut editor = self.edit();
        editor.set_pending(&edits);
        editor.commit_pending();
    }

    ///
    /// Commits a set of edits to this animation
    /// 
    fn commit_edits<I: IntoIterator<Item=AnimationEdit>>(&self, edits: I) {
        let edits: Vec<AnimationEdit> = edits.into_iter().collect();

        // The animation editor performs the edits (via the MutableAnimation implementation)
        let editor = AnimationEditor::new();

        // Send the edits to the edit log
        self.db.insert_edits(edits.iter().cloned());

        // Perform the edits
        let mut mutable = self.db.edit();
        editor.perform(&mut *mutable, edits);
    }
}

impl Animation for SqliteAnimation {
    #[inline]
    fn size(&self) -> (f64, f64) {
        self.db.size()
    }

    #[inline]
    fn get_layer_ids(&self) -> Vec<u64> {
        self.db.get_layer_ids()
    }

    fn duration(&self) -> Duration {
        // TODO
        Duration::from_millis(120 * 1000)
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Reader<'a, Layer>> {
        // Try to retrieve the layer from the editor
        let layer = self.db.get_layer_with_id(layer_id);

        // Turn into a reader if it exists
        let layer = layer.map(|layer| {
            let boxed: Box<Layer> = Box::new(layer);
            boxed
        });

        layer.map(|layer| Reader::new(layer))
    }

    fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>> {
        self.db.get_log()
    }

    fn edit<'a>(&'a self) -> Editor<'a, PendingEditLog<AnimationEdit>> {
        // Create an edit log that will commit to this object's log
        let edit_log = InMemoryPendingLog::new(move |edits| self.commit_edits(edits));

        // Turn it into an editor
        let edit_log: Box<'a+PendingEditLog<AnimationEdit>> = Box::new(edit_log);
        Editor::new(edit_log)
    }

    fn edit_layer<'a>(&'a self, layer_id: u64) -> Editor<'a, PendingEditLog<LayerEdit>> {
        // Create an edit log that will commit to this object's log
        let edit_log = InMemoryPendingLog::new(move |edits| {
            let edits = edits.into_iter()
                .map(|edit| AnimationEdit::Layer(layer_id, edit));

            self.commit_edits(edits)
        });

        // Turn it into an editor
        let edit_log: Box<'a+PendingEditLog<LayerEdit>> = Box::new(edit_log);
        Editor::new(edit_log)
    }
}
