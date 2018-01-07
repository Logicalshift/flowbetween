use super::super::viewmodel::*;

use animation::*;
use canvas::*;

///
/// Information that is passed to the tools for particular actions
/// 
pub struct ToolModel<'a, Anim: 'a+Animation> {
    /// The canvas that we are drawing on
    pub canvas: &'a Canvas,

    /// The animation view model for this animation
    pub anim_view_model: &'a AnimationViewModel<Anim>,

    /// The currently selected layer
    pub selected_layer: Reader<'a, Layer>,

    /// The layer ID of the currently selected layer in the canvas
    pub canvas_layer_id: u32
}