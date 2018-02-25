use super::super::viewmodel::*;

use animation::*;
use canvas::*;

use std::time::Duration;

///
/// Information that is passed to the tools for particular actions
/// 
pub struct ToolModel<'a, Anim: 'a+Animation> {
    /// The current timeline time
    pub current_time: Duration,

    /// The canvas that we are drawing on
    pub canvas: &'a Canvas,

    /// The animation view model for this animation
    pub anim_view_model: &'a AnimationViewModel<Anim>,

    /// The ID of the currently selected layer in the animation
    pub selected_layer_id: u64,

    /// The layer ID of the currently selected layer in the canvas
    pub canvas_layer_id: u32,
}
