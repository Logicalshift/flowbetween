use super::super::viewmodel::*;

use animation::*;
use ui::canvas::*;

///
/// Information that is passed to the tools for particular actions
/// 
pub struct ToolModel<'a, Anim: 'a+Animation> {
    /// The canvas that we are drawing on
    canvas: &'a Canvas,

    /// The animation view model for this animation
    anim_view_model: &'a AnimationViewModel<Anim>
}