use crate::model::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;
use flo_canvas_animation::description::*;

use futures::prelude::*;
use futures::channel::mpsc;
use strum::{IntoEnumIterator};

use std::ops::{Deref};
use std::str::{FromStr};
use std::sync::*;

///
/// The actions that are possible for the animation sidebar
///
#[derive(Clone, Debug, PartialEq)]
enum AnimationAction {
    /// Sets the base animation type
    SetBaseAnimationType(BaseAnimationType),

    /// Tick event
    Tick,

    /// Unrecognised action occurred
    Unknown(String)
}

///
/// The 'base' animation for an animation region
///
#[derive(Clone, Debug, PartialEq, Display, EnumString, EnumIter)]
enum BaseAnimationType {
    /// Frame-by-frame animations (later drawings replace the existing ones)
    FrameByFrame,

    /// Build over time animations (later drawings are added to the existing ones, the default behaviour if no animation is defined)
    BuildOverTime,

    /// The first frame is always redrawn and future frames are added to it
    BuildOnFirstFrame,
}

///
/// Wrapper structure used to bind a selected animation element
///
#[derive(Clone, Debug)]
pub struct SelectedAnimationElement(pub Arc<AnimationElement>);

impl PartialEq for SelectedAnimationElement {
    fn eq(&self, other: &SelectedAnimationElement) -> bool { self.0.id() == other.0.id() }
}

impl Deref for SelectedAnimationElement {
    type Target = AnimationElement;

    fn deref(&self) -> &AnimationElement { &*self.0 }
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
                    animation_elements.push(SelectedAnimationElement(Arc::new(animation_region)));
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

impl BaseAnimationType {
    ///
    /// Generates a description of this animation type
    ///
    fn description(&self) -> &str {
        use self::BaseAnimationType::*;

        match self {
            FrameByFrame        => { "Frame-by-frame" }
            BuildOverTime       => { "Build over time" }
            BuildOnFirstFrame   => { "Draw over initial frame" }
        }
    }
}

///
/// Returns the base animation type for an effect description
///
fn base_animation_type(description: &EffectDescription) -> BaseAnimationType {
    use self::EffectDescription::*;

    match description {
        // The 'frame by frame' animation types override the usual 'build over time' effect
        FrameByFrameReplaceWhole    => BaseAnimationType::FrameByFrame,
        FrameByFrameAddToInitial    => BaseAnimationType::BuildOnFirstFrame,

        // In sequences, the first element determines the base animation type
        Sequence(sequence)          => { if sequence.len() > 0 { base_animation_type(&sequence[0]) } else { BaseAnimationType::BuildOverTime } }
        
        // Embedded effects like repeats or time curves preserve the base animation type of their underlying animation
        Repeat(_length, effect)     => { base_animation_type(&*effect) },
        TimeCurve(_curve, effect)   => { base_animation_type(&*effect) },

        // Other built-in effects mean there's no 'base' type, ie we're using the build over time effect
        Other(_, _)                 |
        Move(_, _)                  |
        FittedTransform(_, _)       |
        StopMotionTransform(_, _)   => BaseAnimationType::BuildOverTime
    }
}

///
/// Creates the binding for the animation sidebar user interface
///
fn animation_sidebar_ui(selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> BindRef<Control> {
    computed(move || {
        use self::Position::*;

        let selected_elements = selected_animation_elements.get();

        if selected_elements.len() == 0 {
            // Control is inactive with 0 selected elements
            Control::empty()
        } else if selected_elements.len() > 1 {
            // Currently can't edit more than one element at once
            Control::empty()
        } else {
            // Single selected element
            let selected_element        = &selected_elements[0];
            let base_animation_type     = base_animation_type(selected_element.description().effect());

            // The list of base animation type choices and the combo-box allowing selection (derived from the enum definit)
            let base_choices            = BaseAnimationType::iter()
                .map(|base_type| Control::label()
                    .with(base_type.description())
                    .with((ActionTrigger::Click, format!("SetBase {}", base_type))))
                .collect::<Vec<_>>();
            let base_combobox           = Control::combo_box()
                .with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(20.0) })
                .with(base_animation_type.description())
                .with(base_choices);

            // Generate the sidebar container
            Control::container()
                .with(Bounds { x1: Start, y1: Start, x2: End, y2: End } )
                .with(ControlAttribute::Padding((8, 8), (8, 8)))
                .with(vec![
                    base_combobox
                ])
        }
    }).into()
}

impl AnimationAction {
    ///
    /// Generates a sidebar action description from t
    ///
    pub fn from_controller_event(event: &ControllerEvent) -> AnimationAction {
        match event {
            ControllerEvent::Action(action_name, _param) => {
                if action_name.starts_with("SetBase ") {
                    // Parse the animation type
                    let base_animation_type = action_name["SetBase ".len()..].to_string();
                    let base_animation_type = BaseAnimationType::from_str(&base_animation_type);

                    // Set base animation type action
                    if let Ok(base_animation_type) = base_animation_type {
                        AnimationAction::SetBaseAnimationType(base_animation_type)
                    } else {
                        AnimationAction::Unknown(action_name.clone())
                    }
                } else {
                    // Action name doesn't match any we know
                    AnimationAction::Unknown(action_name.clone())
                }
            }

            ControllerEvent::Tick => AnimationAction::Tick,
        }
    }
}

///
/// Runs the animation sidebar panel
///
async fn run_animation_sidebar_panel<Anim: 'static+EditableAnimation>(events: ControllerEventStream, _actions: mpsc::Sender<ControllerAction>, _resources: ControllerResources, model: Arc<FloModel<Anim>>) {
    // Convert the events into animation events
    let mut events = events.map(|event| AnimationAction::from_controller_event(&event));

    // Run while there are events pending
    while let Some(event) = events.next().await {
        match event {
            // Tick events are ignored, unknown events are logged
            AnimationAction::Unknown(evt)   => { warn!("Unknown animation sidebar event {}", evt); }
            AnimationAction::Tick           => { }

            AnimationAction::SetBaseAnimationType(new_base_type) => {
                // Set the base animation type for the selected region
                println!("New base type: {:?}", new_base_type);
            }
        }
    }
}

///
/// The Animation panel is used to show an overview of the effects in the currently selected animation element(s)
///
pub fn animation_sidebar_panel<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> SidebarPanel {
    // Create the controller for the panel
    let ui                  = animation_sidebar_ui(selected_animation_elements.clone());
    let model               = model.clone();
    let controller          = ImmediateController::with_ui(ui,
        move |events, actions, resources| run_animation_sidebar_panel(events, actions, resources, model.clone()));

    // The panel is 'active' if there is one or more elements selected
    let is_active           = computed(move || selected_animation_elements.get().len() > 0);

    // Construct the sidebar panel
    SidebarPanel::with_title("Animation")
        .with_active(BindRef::from(is_active))
        .with_height(bind(200.0))
        .with_controller(controller)
}
