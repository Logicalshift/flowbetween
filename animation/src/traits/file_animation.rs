use super::animation::*;

use std::path::Path;

///
/// Trait implemented by animations that can be created from files
///
pub trait FileAnimation : Send+Sync {
    type NewAnimation: EditableAnimation;

    ///
    /// Opens an animation from a file on disk
    ///
    fn open(&self, path: &Path) -> Self::NewAnimation;
}
