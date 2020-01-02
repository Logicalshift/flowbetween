use super::*;

use flo_stream::*;
use flo_animation::*;

use std::sync::*;

impl SqliteAnimation {
    ///
    /// Performs a particular set of edits immediately to this animation
    ///
    pub fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        self.db.perform_edits(edits);
    }
}

impl EditableAnimation for SqliteAnimation {
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.db.create_edit_sink()
    }
}
