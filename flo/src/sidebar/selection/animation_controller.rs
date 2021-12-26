use crate::model::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::channel::mpsc;

use std::ops::{Deref};
use std::sync::*;

///
/// Wrapper structure used to bind a selected animation element
///
#[derive(Clone, Debug)]
pub struct SelectedAnimationElement(pub AnimationElement);

impl PartialEq for SelectedAnimationElement {
    fn eq(&self, other: &SelectedAnimationElement) -> bool { self.0.id() == other.0.id() }
}

impl Deref for SelectedAnimationElement {
    type Target = AnimationElement;

    fn deref(&self) -> &AnimationElement { &self.0 }
}

///
/// Creates a binding that tracks the set of currently selected animation elements
///
pub fn selected_animation_elements<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> BindRef<Vec<SelectedAnimationElement>> {
    let selected_element_ids    = model.selection().selected_elements.clone();
    let frame                   = model.frame().frame.clone();

    let animation_elements = computed(move || {
        let selected_element_ids    = selected_element_ids.get();
        let frame                   = frame.get();

        if let Some(frame) = frame {
            // Fetch the elements from the frame and find 
            let mut animation_elements = vec![];

            for element_id in selected_element_ids.iter() {
                let element = frame.element_with_id(*element_id);
                let element = if let Some(element) = element { element } else { continue; };

                if let Vector::AnimationRegion(animation_region) = element {
                    animation_elements.push(SelectedAnimationElement(animation_region));
                }
            }

            animation_elements
        } else {
            // No frame selected
            vec![]
        }
    });

    BindRef::from(animation_elements)
}

///
/// Creates the binding for the animation sidebar user interface
///
fn animation_sidebar_ui() -> BindRef<Control> {
    computed(move || {
        use self::Position::*;

        Control::container()
            .with(Bounds { x1: Start, y1: Start, x2: End, y2: End } )
            .with(ControlAttribute::Padding((8, 8), (8, 8)))
            .with(vec![
                Control::combo_box()
                    .with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(20.0) })
                    .with("Frame-by-frame")
                    .with(vec![
                        Control::label().with("Frame-by-frame").with((ActionTrigger::Click, "StyleFrameByFrame")),
                        Control::label().with("Build over time").with((ActionTrigger::Click, "StyleBuildOverTime")),
                    ])
            ])
    }).into()
}

///
/// Runs the animation sidebar panel
///
pub async fn run_animation_sidebar_panel(_events: ControllerEventStream, _actions: mpsc::Sender<ControllerAction>, _resources: ControllerResources) {
    // TODO
}

///
/// The Animation panel is used to show an overview of the effects in the currently selected animation element(s)
///
pub fn animation_sidebar_panel<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> SidebarPanel {
    // Create the controller for the panel
    let ui                  = animation_sidebar_ui();
    let controller          = ImmediateController::with_ui(ui,
        move |events, actions, resources| run_animation_sidebar_panel(events, actions, resources));

    // The panel is 'active' if there is one or more elements selected
    let selected_elements   = model.selection().selected_elements.clone();
    let is_active           = computed(move || selected_elements.get().len() > 0);

    // Construct the sidebar panel
    SidebarPanel::with_title("Animation")
        .with_active(BindRef::from(is_active))
        .with_height(bind(200.0))
        .with_controller(controller)
}
