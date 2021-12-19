use crate::model::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::channel::mpsc;

use std::sync::*;

///
/// Runs the animation sidebar panel
///
pub async fn run_animation_sidebar_panel(_events: ControllerEventStream, _actions: mpsc::Sender<ControllerAction>, _resources: ControllerResources) {
    // TODO
}

///
/// The Animation panel is used to show an overview of the effects in the currently selected animation element(s)
///
pub fn animation_sidebar_panel<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> SidebarPanel {
    // Create the controller for the panel
    let controller          = ImmediateController::empty(move |events, actions, resources| run_animation_sidebar_panel(events, actions, resources));

    // The panel is 'active' if there is one or more elements selected
    let selected_elements   = model.selection().selected_elements.clone();
    let is_active           = computed(move || selected_elements.get().len() > 0);

    // Construct the sidebar panel
    SidebarPanel::with_title("Animation")
        .with_active(BindRef::from(is_active))
        .with_controller(controller)
}
