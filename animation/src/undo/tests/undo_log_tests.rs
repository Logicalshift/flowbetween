use crate::undo::undo_log::*;
use crate::traits::*;

use std::sync::*;

#[test]
fn commits_final_finish_action() {
    let mut log = UndoLog::new();

    assert!(log.undo_depth() == 0);

    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::AddNewLayer(0)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));

    // Two entries: the action we just finished and the new action we're carrying out
    assert!(log.undo_depth() == 2);
}

#[test]
fn does_not_commit_empty_action() {
    let mut log = UndoLog::new();

    assert!(log.undo_depth() == 0);

    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::AddNewLayer(0)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));
    log.retire(RetiredEdit::new(Arc::new(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]), Arc::new(vec![])));

    // Still two entries: the extra 'FinishAction' is not treated as it's own undo action
    assert!(log.undo_depth() == 2);
}