use super::animation::*;

use std::path::Path;

///
/// Trait implemented by animations that can be created from files
///
pub trait FileAnimation : Animation {
    ///
    /// Opens an animation from a file on disk
    ///
    fn open(path: &Path) -> Self;
}
