use crate::model::*;
use crate::style::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_stream::*;
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

    /// Unrecognised action occurred
    Unknown(String)
}

///
/// The 'base' animation for an animation region
///
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, EnumIter)]
enum BaseAnimationType {
    /// Build over time animations (later drawings are added to the existing ones, the default behaviour if no animation is defined)
    BuildOverTime,

    /// Frame-by-frame animations (later drawings replace the existing ones)
    FrameByFrame,

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
    let edit_counter            = model.frame_update_count();

    let animation_elements = computed(move || {
        // Refresh whenever the edit counter changes
        edit_counter.get();

        // Read the elements from the current frame
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

// TODO: the 'update base description' and probably the list of base animation types might be best moved to the EffectDescription definition so they can be used in other contexts if needed

///
/// Creates a new effect description for a new base animation type
///
fn update_effect_animation_type(old_description: &EffectDescription, new_base_type: BaseAnimationType) -> EffectDescription {
    use self::EffectDescription::*;

    // Work out the new base description element
    let new_description = match new_base_type {
        BaseAnimationType::FrameByFrame         => Some(FrameByFrameReplaceWhole),
        BaseAnimationType::BuildOnFirstFrame    => Some(FrameByFrameAddToInitial),
        BaseAnimationType::BuildOverTime        => None
    };

    match old_description {
        // Basic frame-by-frame items are replaced with sequences
        FrameByFrameReplaceWhole    |
        FrameByFrameAddToInitial    => Sequence(new_description.into_iter().collect()),

        // Sequences update the first element (or insert a new first element if there's no base type there)
        Sequence(sequence)          => {
            if sequence.len() == 0 {
                // Empty sequence is just replaced with the new base description
                Sequence(new_description.into_iter().collect())
            } else {
                // First sequence element is replaced if it's already a base animation type, otherwise the base type is added to the start of the sequence
                match sequence[0] {
                    FrameByFrameReplaceWhole | FrameByFrameAddToInitial => Sequence(new_description.into_iter().chain(sequence.iter().skip(1).cloned()).collect()),
                    _                                                   => Sequence(new_description.into_iter().chain(sequence.iter().cloned()).collect())
                }
            }
        }

        // Embedded effects recurse
        Repeat(length, effect)      => { Repeat(*length, Box::new(update_effect_animation_type(&*effect, new_base_type))) },
        TimeCurve(curve, effect)    => { TimeCurve(curve.clone(), Box::new(update_effect_animation_type(&*effect, new_base_type))) },

        // Other effects are unaffected
        Other(_, _)                 |
        Move(_, _)                  |
        FittedTransform(_, _)       |
        StopMotionTransform(_, _)   => old_description.clone()
    }
}

///
/// Creates the editing instructions to update the base animation type of an animation element
///
fn set_base_animation_type(animation_element: &AnimationElement, new_base_type: BaseAnimationType) -> Vec<AnimationEdit> {
    // Gather information
    let element_id      = animation_element.id();
    let region          = animation_element.description().region().clone();
    let effect          = animation_element.description().effect();

    // Update the description for the animation element
    let new_effect      = update_effect_animation_type(effect, new_base_type);
    let new_description = RegionDescription(region, new_effect);

    // Edit the model
    vec![
        AnimationEdit::Element(vec![element_id], ElementEdit::SetAnimationDescription(new_description))
    ]
}

///
/// Creates the binding for the animation sidebar user interface
///
fn animation_sidebar_ui<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> BindRef<Control> {
    // Create a binding for the base animation type for the selected element
    let edit_counter                    = model.frame_update_count();
    let selected_animation_elements_2   = selected_animation_elements.clone();
    let selected_base_anim_type         = computed(move || { 
        // We re-check when the edit counter changes, or the selected elements change
        edit_counter.get();
        let selected_elements = selected_animation_elements_2.get();

        // We can't currently update the base type if there are multiple selected elements or no selected elements
        if selected_elements.len() == 0 {
            // Default to frame-by-frame for no elements
            BaseAnimationType::FrameByFrame
        } else if selected_elements.len() > 1 {
            // Default to frame-by-frame for multiple elements
            BaseAnimationType::FrameByFrame
        } else {
            // Only one element selected
            let selected_element        = &selected_elements[0];
            let element_animation_type  = base_animation_type(selected_element.description().effect());

            element_animation_type
        }
    });

    // Generate the UI for the animation panel
    computed(move || {
        let selected_elements = selected_animation_elements.get();

        if selected_elements.len() == 0 {
            // Control is inactive with 0 selected elements
            Control::empty()
        } else if selected_elements.len() > 1 {
            // Currently can't edit more than one element at once
            Control::empty()
        } else {
            // Single selected element
            let element_animation_type  = selected_base_anim_type.get();

            // The list of base animation type choices and the combo-box allowing selection (derived from the enum definit)
            let base_choices            = BaseAnimationType::iter()
                .map(|base_type| if base_type != element_animation_type {
                    Control::label()
                        .with(base_type.description())
                        .with((ActionTrigger::Click, format!("SetBase {}", base_type)))
                    } else {
                        // No action is generated for the element that is already selected
                        Control::label()
                            .with(base_type.description())
                    })
                .collect::<Vec<_>>();
            let base_combobox           = Control::combo_box()
                .with(Bounds::next_vert(20.0))
                .with(element_animation_type.description())
                .with(Hover::Tooltip("How this region is animated".to_string()))
                .with(base_choices);

            // The effect selector
            let effect_selector = Control::container()
                .with(Bounds::stretch_vert(1.0))
                .with(ControlAttribute::Padding((1, 1), (1, 1)))
                .with(Appearance::Background(CONTROL_BORDER))
                .with(vec![
                    Control::container()
                        .with(Bounds::fill_all())
                        .with(Appearance::Background(CONTROL_BACKGROUND))
                ]);

            // The action buttons for adding new effects
            let action_buttons = Control::container()
                .with(Bounds::next_vert(20.0))
                .with(Font::Size(9.0))
                .with(vec![
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0)),
                    Control::container()
                        .with(Hint::Class("button-group".to_string()))
                        .with(Bounds::next_horiz(22.0*4.0))
                        .with(vec![
                            Control::button()
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(Bounds::next_horiz(20.0)),
                            Control::button()
                                .with(Bounds::next_horiz(20.0)),
                            Control::button()
                                .with(Bounds::next_horiz(22.0)),
                        ])
                ]);

            // Generate the sidebar container
            Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((8, 8), (8, 8)))
                .with(vec![
                    base_combobox,

                    Control::empty()
                        .with(Bounds::next_vert(6.0)),

                    Control::label()
                        .with("Effects")
                        .with(Font::Size(10.0))
                        .with(Appearance::Foreground(DEFAULT_TEXT))
                        .with(Bounds::next_vert(12.0)),
                    Control::empty()
                        .with(Bounds::next_vert(3.0)),
                    effect_selector,

                    Control::empty()
                        .with(Bounds::next_vert(3.0)),

                    action_buttons
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
        }
    }
}

///
/// Runs the animation sidebar panel
///
async fn run_animation_sidebar_panel<Anim: 'static+EditableAnimation>(events: ControllerEventStream, _actions: mpsc::Sender<ControllerAction>, _resources: ControllerResources, model: Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) {
    // Convert the events into animation events
    let mut events = events.map(|event| AnimationAction::from_controller_event(&event));

    // Run while there are events pending
    while let Some(event) = events.next().await {
        match event {
            // Unknown events are logged
            AnimationAction::Unknown(evt)   => { warn!("Unknown animation sidebar event {}", evt); }

            AnimationAction::SetBaseAnimationType(new_base_type) => {
                // Set the base animation type for the selected region
                let edits       = selected_animation_elements.get().iter()
                    .flat_map(|selected_element| set_base_animation_type(selected_element, new_base_type))
                    .collect();

                // Send the edits to the animation and wait for them to be received
                let mut editor  = model.edit();
                editor.publish(Arc::new(edits)).await;
                editor.when_empty().await;
            }
        }
    }
}

///
/// The Animation panel is used to show an overview of the effects in the currently selected animation element(s)
///
pub fn animation_sidebar_panel<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> SidebarPanel {
    // Create the controller for the panel
    let ui                  = animation_sidebar_ui(model, selected_animation_elements.clone());
    let model               = model.clone();
    let selected_elem_clone = selected_animation_elements.clone();
    let controller          = ImmediateController::with_ui(ui,
        move |events, actions, resources| run_animation_sidebar_panel(events, actions, resources, model.clone(), selected_elem_clone.clone()));

    // The panel is 'active' if there is one or more elements selected
    let is_active           = computed(move || selected_animation_elements.get().len() > 0);

    // Construct the sidebar panel
    SidebarPanel::with_title("Animation")
        .with_active(BindRef::from(is_active))
        .with_height(bind(200.0))
        .with_controller(controller)
}
