use crate::model::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;
use flo_canvas_animation::description::*;

use std::sync::*;

///
/// Creates the binding that indicates if the repeat sidebar panel is active or not
///
fn repeat_panel_active<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> BindRef<bool> {
    let selected_sub_effect = model.selection().selected_sub_effect.clone();

    computed(move || {
        if let Some((_elem_id, subeffect)) = selected_sub_effect.get() {
            // Sub-effect must be a repeat element
            match subeffect.effect_description() {
                EffectDescription::Repeat(_, _) => true,
                _                               => false
            }
        } else {
            // No sub-effect selected
            false
        }
    }).into()
}

///
/// Creates the 'repeat effect' animation sidebar panel
///
pub fn animation_repeat_sidebar_panel<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> SidebarPanel {
    // Set up the model
    let model       = Arc::clone(model);
    let is_active   = repeat_panel_active(&model);

    // Create a new immediate controller
    let controller = ImmediateController::empty(move |actions, events, resources| {
        async move {

        }
    });

    SidebarPanel::with_title("Animation: Repeat")
        .with_active(is_active)
        .with_controller(controller)
        .with_height(bind(128.0))
}
