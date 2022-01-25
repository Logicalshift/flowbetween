use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::document_settings::*;

use flo_binding::*;
use flo_animation::*;

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
/// Returns the updates for the rope of selection panels
///
pub fn document_panels<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> Vec<SidebarPanel> {
    let model = Arc::clone(model);

    vec![
        document_settings_panel(&model)
    ]
}