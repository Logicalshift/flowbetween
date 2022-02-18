use crate::traits::*;

use std::mem;
use std::sync::*;
use std::ops::{Deref, DerefMut};

///
/// A reversed set of animation edits (to keep the reversed edits separate from the main set)
///
#[derive(Clone, Debug)]
pub struct ReversedEdits(pub Vec<AnimationEdit>);

impl ReversedEdits {
    ///
    /// Creates a new reversed edits structure, where the intention is to populate it later on
    ///
    pub fn new() -> ReversedEdits {
        ReversedEdits(vec![])
    }

    ///
    /// Creates a new reversed edits structure, where it's expected to be empty
    ///
    pub fn empty() -> ReversedEdits {
        ReversedEdits(vec![])
    }

    ///
    /// As for `new()` except a placeholder for an edit that does not have a reversal yet
    ///
    pub fn unimplemented() -> ReversedEdits {
        Self::empty()
    }

    ///
    /// Adds a set of reversed edits to the start of this list of edits
    ///
    pub fn add_to_start(&mut self, edits: ReversedEdits) {
        // Move the 'new' edits so they're the start of a new vector
        let mut edits = edits.0;
        mem::swap(&mut self.0, &mut edits);

        // Append the edits that were originally in this object
        self.0.extend(edits);
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
