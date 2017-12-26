use super::super::viewmodel::*;

use animation::*;
use ui::canvas::*;

use std::sync::*;

///
/// Information that is passed to the tools for particular actions
/// 
pub struct ToolModel<'a, Anim: 'a+Animation> {
    /// The canvas that we are drawing on
    pub canvas: &'a Canvas,

    /// The animation view model for this animation
    pub anim_view_model: &'a AnimationViewModel<Anim>,

    /// The currently selected layer
    pub selected_layer: Arc<Layer>,

    /// The layer ID of the currently selected layer in the canvas
    pub selected_layer_id: u32
}