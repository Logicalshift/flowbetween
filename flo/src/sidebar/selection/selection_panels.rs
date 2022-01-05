use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::selection::animation_controller::*;
use crate::sidebar::selection::animation_repeat::*;

use flo_rope::*;
use flo_binding::*;
use flo_animation::*;
use flo_canvas_animation::description::*;

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
    let selected_sub_effect         = model.selection().selected_sub_effect.clone();
    let animation_panel             = animation_sidebar_panel(&model, selected_animation_elements.clone());
    let anim_repeat_panel           = animation_repeat_sidebar_panel(&model);

    // Create a binding that describes the selection panels that are being displayed
    let selection_panels            = RopeBinding::computed_difference(move || {
        let mut panels = vec![];

        // The main animation panel shows up if the user has selected a single animation region
        let animation_elements  = selected_animation_elements.get();

        if animation_elements.len() == 1 {
            // User has picked exactly one animation element
            panels.push(animation_panel.clone());
        }

        // If the user has an animation element and subelement selected, then add the appropriate panel for editing that element
        if animation_elements.len() > 0 {
            if let Some((_, subeffect)) = selected_sub_effect.get() {
                match subeffect.effect_type() {
                    SubEffectType::Other                => { }
                    SubEffectType::Repeat               => { panels.push(anim_repeat_panel.clone()); }
                    SubEffectType::TimeCurve            => { }
                    SubEffectType::LinearPosition       => { }
                    SubEffectType::TransformPosition    => { }
                }
            }
        }

        panels
    });

    // Follow the changes to the set of selection panels
    selection_panels.follow_changes_retained()
}
