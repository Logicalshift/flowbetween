use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::selection::animation_controller::*;

use flo_rope::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream;

use std::sync::*;

///
/// Returns the updates for the rope of selection panels
///
pub fn selection_panels<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=RopeAction<SidebarPanel, ()>> {
    animation_selection_panels(model)
}

///
/// Returns the updates for the rope of selection panels relating to the animation
///
pub fn animation_selection_panels<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Stream<Item=RopeAction<SidebarPanel, ()>> {
    let model           = Arc::clone(model);
    let animation_panel = animation_sidebar_panel(&model);

    RopeBinding::computed(move || {
        let selected_element_ids    = model.selection().selected_elements.get();
        let frame                   = model.frame().frame.get();

        if let Some(frame) = frame {
            // Fetch the elements from the frame and find 
            let mut animation_elements = vec![];

            for element_id in selected_element_ids.iter() {
                let element = frame.element_with_id(*element_id);
                let element = if let Some(element) = element { element } else { continue; };

                if let Vector::AnimationRegion(animation_region) = element {
                    animation_elements.push(animation_region);
                }
            }

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
        } else {
            // No frame
            vec![]
        }
    }).follow_changes()
}
