use super::*;

use futures::*;
use futures::executor;
use animation::*;

impl SqliteAnimation {
    ///
    /// Performs a particular set of edits immediately to this animation
    ///
    pub fn perform_edits(&mut self, edits: Vec<AnimationEdit>) {
        let mut sink = executor::spawn(self.db.create_edit_sink());

        sink.wait_send(edits).unwrap();
        sink.wait_flush().unwrap();
    }
}

impl EditableAnimation for SqliteAnimation {
    fn edit<'a>(&'a self) -> Box<'a+Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>> {
        self.db.create_edit_sink()
    }
}
