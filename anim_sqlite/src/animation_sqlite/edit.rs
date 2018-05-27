use super::*;

use futures::*;
use animation::*;

impl EditableAnimation for SqliteAnimation {
    fn edit<'a>(&'a self) -> Box<'a+Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>> {
        self.db.create_edit_sink()
    }
}