use super::*;

use futures::*;
use futures::executor;
use animation::*;

impl SqliteAnimation {
    ///
    /// Performs a particular set of edits immediately to this animation
    ///
    pub fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        let mut sink = executor::spawn(self.db.create_edit_sink());

        sink.wait_send(edits).unwrap();
        sink.wait_flush().unwrap();
    }
}

impl EditableAnimation for SqliteAnimation {
    fn edit(&self) -> Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send> {
        self.db.create_edit_sink()
    }
}
