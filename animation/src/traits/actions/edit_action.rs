use super::super::edit::*;
use super::super::animation::*;

///
/// The edit action trait is implemented by items that represent a more complicated editing
/// action that can be broken down into a set of animation edits with reference to the
/// animation that they should be performed upon.
///
pub trait EditAction {
    ///
    /// Converts this edit action into a set of animation edits for a particular animation
    ///
    fn to_animation_edits<Anim: EditableAnimation>(&self, animation: &Anim) -> Vec<AnimationEdit>;
}

impl EditAction for AnimationEdit {
    #[inline]
    fn to_animation_edits<Anim: EditableAnimation>(&self, _animation: &Anim) -> Vec<AnimationEdit> {
        vec![self.clone()]
    }
}
