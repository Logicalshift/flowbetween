use super::super::traits::*;

use std::path::Path;

///
/// Represents a function that loads an animation
///
pub struct AnimationLoader<TFn, TAnim>(TFn)
where TFn: Fn(&Path) -> TAnim,
TAnim: Animation;

impl<TFn, TAnim> FileAnimation for AnimationLoader<TFn, TAnim>
where TFn: Fn(&Path) -> TAnim,
TAnim: Animation {
    type NewAnimation = TAnim;

    ///
    /// Opens an animation from a file on disk
    ///
    fn open(&self, path: &Path) -> Self::NewAnimation {
        self.0(path)
    }
}
