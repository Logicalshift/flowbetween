use crate::undo::undo_log::*;
use crate::traits::*;

use std::sync::*;

#[test]
fn commits_final_finish_action() {
    let mut log = UndoLog::new();

    assert!(log.undo_depth() == 0);

    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::AddNewLayer(0)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));

    // Should create a single entry
    assert!(log.undo_depth() == 1);
}

#[test]
fn does_not_commit_empty_action() {
    let mut log = UndoLog::new();

    assert!(log.undo_depth() == 0);

    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::AddNewLayer(0)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));

    // The last 'FinishAction' should not create a new entry
    assert!(log.undo_depth() == 1);
}