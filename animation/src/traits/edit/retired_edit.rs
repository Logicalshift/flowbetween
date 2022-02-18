use super::animation_edit::*;

use std::sync::*;

///
/// Contains a sequence of edits that are being 'retired' from processing
///
#[derive(Clone, PartialEq, Debug)]
pub struct RetiredEdit {
    /// The edits that were committed to the animation
    committed:  Arc<Vec<AnimationEdit>>,

    /// The actions that will reverse these edits
    reverse:    Arc<Vec<AnimationEdit>>,
}

impl RetiredEdit {
    ///
    /// Creates a new 'retired edit' structure with the specified edits committed and a list of undo actions
    ///
    pub fn new(committed: Arc<Vec<AnimationEdit>>, reverse: Arc<Vec<AnimationEdit>>) -> RetiredEdit {
        RetiredEdit {
            committed:  Arc::clone(&committed),
            reverse:    Arc::clone(&reverse),
        }
    }

    ///
    /// Returns the list of edits that were committed to the animation
    ///
    pub fn committed_edits(&self) -> Arc<Vec<AnimationEdit>> {
        Arc::clone(&self.committed)
    }

    ///
    /// Returns the list of edits that will reverse what was done by the committed edits
    ///
    pub fn reverse_edits(&self) -> Arc<Vec<AnimationEdit>> {
        Arc::clone(&self.reverse)
    }
}
