use super::super::traits::*;

use std::path::Path;

///
/// Represents a function that loads an animation
///
pub struct AnimationLoader<TFn, TAnim>(pub TFn)
where TFn: Send+Sync+Fn(&Path) -> TAnim,
TAnim: Animation;

impl<TFn, TAnim> FileAnimation for AnimationLoader<TFn, TAnim>
where TFn: Send+Sync+Fn(&Path) -> TAnim,
TAnim: EditableAnimation {
    type NewAnimation = TAnim;

    ///
    /// Opens an animation from a file on disk
    ///
    fn open(&self, path: &Path) -> Self::NewAnimation {
        self.0(path)
    }
}
