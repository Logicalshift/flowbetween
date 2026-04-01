use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::layer_settings::*;
use crate::sidebar::document_settings::*;

use flo_rope::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::sync::*;

///
/// Creates the document settings sidebar panel
///
pub fn document_settings_panel<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> SidebarPanel {
    let height = bind(100.0);

    SidebarPanel::with_title("Document")
        .with_controller(document_settings_controller(model, height.clone()))
        .with_height(height)
}

///
/// Creates the layer settings sidebar panel
///
pub fn layer_settings_panel<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> SidebarPanel {
    let height = bind(100.0);

    SidebarPanel::with_title("Layer")
        .with_controller(layer_settings_controller(model, height.clone()))
        .with_height(height)
}
///
/// Returns the updates for the rope of selection panels
///
pub fn document_panels<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=RopeAction<SidebarPanel, ()>>  {
    // Set up the model and load the panels
    let model                   = Arc::clone(model);
    let layer_settings_panel    = layer_settings_panel(&model);
    let document_settings_panel = document_settings_panel(&model);

    // Create a rope binding for the panels
    let document_panels = RopeBinding::computed_difference(move || {
        let mut panels      = vec![];
        let selected_layer  = model.timeline().selected_layer.get();

        if selected_layer.is_some() {
            panels.push(layer_settings_panel.clone());
        }
        panels.push(document_settings_panel.clone());

        panels
    });

    // Follow the changes to the set of selection panels (retaining the rope so the stream doesn't immediately end)
    document_panels.follow_changes_retained()
}
