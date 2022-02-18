use crate::traits::*;

use std::sync::*;
use std::ops::{Deref, DerefMut};

///
/// A reversed set of animation edits (to keep the reversed edits separate from the main set)
///
#[derive(Clone, Debug)]
pub struct ReversedEdits(pub Vec<AnimationEdit>);

impl ReversedEdits {
    pub fn new() -> ReversedEdits {
        ReversedEdits(vec![])
    }
}

impl From<Vec<AnimationEdit>> for ReversedEdits {
    fn from(edits: Vec<AnimationEdit>) -> ReversedEdits {
        ReversedEdits(edits)
    }
}

impl Into<Arc<Vec<AnimationEdit>>> for ReversedEdits {
    fn into(self) -> Arc<Vec<AnimationEdit>> {
        Arc::new(self.0)
    }
}

impl Deref for ReversedEdits {
    type Target = Vec<AnimationEdit>;

    #[inline]
    fn deref(&self) -> &Vec<AnimationEdit> {
        &self.0
    }
}

impl DerefMut for ReversedEdits {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<AnimationEdit> {
        &mut self.0
    }
}
