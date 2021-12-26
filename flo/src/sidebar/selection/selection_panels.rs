use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::selection::animation_controller::*;

use flo_rope::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::sync::*;

///
/// Returns the updates for the rope of selection panels
///
pub fn selection_panels<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=RopeAction<SidebarPanel, ()>> {
    let animation_panels = animation_selection_panels(model);

    animation_panels
}

///
/// Returns the updates for the rope of selection panels relating to the animation
///
pub fn animation_selection_panels<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=RopeAction<SidebarPanel, ()>> {
    // Create the model
    let model                       = Arc::clone(model);

    // Create the panels
    let selected_animation_elements = selected_animation_elements(&model);
    let animation_panel             = animation_sidebar_panel(&model, selected_animation_elements.clone());

    // Create a binding that describes the selection panels that are being displayed
    let selection_panels            = RopeBinding::computed_difference(move || {
        let animation_elements  = selected_animation_elements.get();

        if animation_elements.len() == 0 {
            // User has picked no animation elements
            vec![]
        } else if animation_elements.len() == 1 {
            // User has picked exactly one animation element
            vec![animation_panel.clone()]
        } else {
            // User has picked multiple animation elements
            vec![]
        }
    });

    // Follow the changes to the set of selection panels
    selection_panels.follow_changes_retained()
}
