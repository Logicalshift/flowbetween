use super::*;

use flo_stream::*;
use flo_animation::*;

use futures::executor;

use std::sync::*;

impl SqliteAnimation {
    ///
    /// Performs a particular set of edits immediately to this animation
    ///
    pub fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        let mut publisher = self.db.create_edit_sink();

        executor::block_on(async {
            publisher.publish(Arc::new(edits)).await;
            publisher.when_empty().await;
        })
    }
}

impl EditableAnimation for SqliteAnimation {
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.db.create_edit_sink()
    }
}
