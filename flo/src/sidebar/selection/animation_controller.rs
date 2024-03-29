use crate::model::*;
use crate::style::*;
use crate::sidebar::panel::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;
use flo_canvas_animation::description::*;

use futures::prelude::*;
use futures::stream;
use futures::channel::mpsc;
use strum::{IntoEnumIterator};
use uuid::*;
use smallvec::*;

use std::ops::{Deref};
use std::str::{FromStr};
use std::sync::*;

///
/// Describes an effect that can be added to an existing effect
///
#[derive(Clone, Debug, PartialEq)]
struct AvailableEffect {
    /// The ID of this effect (a unique string)
    id: String,

    /// The name of this effect
    name: String,

    /// The sequence of element edits required to add this effect to an existing animation element
    create: SmallVec<[ElementEdit; 2]>
}

///
/// Bindings used within the animation controller
///
struct AnimationControllerModel {
    /// The selected set of animation elements
    selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>,

    /// The list of effects that can be selected between (the selected effect is in the main selection model)
    effects: BindRef<Vec<(ElementId, Arc<SubEffectDescription>)>>,

    /// Binding used to open the 'add new effect' popup
    add_popup_open: Binding<bool>,

    /// A list of effects that can be added to the current animation element
    available_effects: BindRef<Vec<AvailableEffect>>,
}

///
/// The actions that are possible for the animation sidebar
///
#[derive(Clone, Debug, PartialEq)]
enum AnimationAction {
    /// Sets the base animation type
    SetBaseAnimationType(BaseAnimationType),

    /// Selects the sub-effect at the specified index
    SelectEffect(usize),

    /// User clicked the 'add effect' button
    OpenNewEffectPopup,

    /// The set of selected elements has changed
    SelectionChanged,

    /// The canvas has been updated
    CanvasUpdated,

    /// Unrecognised action occurred
    Unknown(String)
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

impl AvailableEffect {
    ///
    /// Creates a new available effect with a unique ID
    ///
    pub fn new(name: &str, create: SmallVec<[ElementEdit; 2]>) -> AvailableEffect {
        let id = Uuid::new_v4().to_simple().to_string();

        AvailableEffect {
            id:     id,
            name:   name.to_string(),
            create: create
        }
    }
}

///
/// The default list of effects that can be added to an animation
///
fn default_effects() -> Vec<AvailableEffect> {
    vec![
        AvailableEffect::new("Repeat",      smallvec![ElementEdit::AddAnimationEffect(SubEffectType::Repeat)]),
        AvailableEffect::new("Time Curve",  smallvec![ElementEdit::AddAnimationEffect(SubEffectType::TimeCurve)]),
    ]
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

///
/// Creates the UI and controller that runs the 'new effect' popup
///
fn new_effect_controller<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, anim_model: &Arc<AnimationControllerModel>) -> impl Controller {
    let effect_height       = 20.0;

    // UI to display the available effects
    let available_effects   = anim_model.available_effects.clone();
    let new_effects_ui      = computed(move || {
        let title_controls      = vec![
            Control::container()
                .with(Bounds::next_vert(16.0))
                .with(ControlAttribute::Padding((8, 2), (8, 0)))
                .with(Font::Size(11.0))
                .with(vec![
                    Control::label()
                        .with(Bounds::fill_all())
                        .with(Font::Size(11.0))
                        .with(TextAlign::Center)
                        .with(Font::Weight(FontWeight::Light))
                        .with("Add new effect:")
                ]),
            Control::empty()
                .with(Bounds::next_vert(4.0)),
            Control::empty()
                .with(Bounds::next_vert(1.0))
                .with(Appearance::Background(CONTROL_BORDER)),
        ];

        // Create a list of controls for selecting an effect
        let available_effects   = available_effects.get();
        let effect_controls     = available_effects.into_iter()
            .flat_map(|effect| {
                vec![
                    Control::container()
                        .with(Bounds::next_vert(effect_height))
                        .with(ControlAttribute::Padding((8, 0), (8, 0)))
                        .with((ActionTrigger::Click, effect.id.as_str()))
                        .with(vec![
                            Control::label()
                                .with(Bounds::fill_all())
                                .with(effect.name.as_str())
                                .with(TextAlign::Left)
                                .with((ActionTrigger::Click, effect.id.as_str()))
                        ]),
                    Control::empty()
                        .with(Bounds::next_vert(1.0))
                        .with(Appearance::Background(CONTROL_BORDER)),
                ]
            });

        let effect_controls     = title_controls.into_iter()
            .chain(effect_controls)
            .collect::<Vec<_>>();

        // Put into a scrolling container
        Control::container()
            .with(Appearance::Background(CONTROL_BORDER))
            .with(ControlAttribute::Padding((1, 1), (1, 1)))
            .with(Bounds::fill_all())
            .with(vec![
                Control::scrolling_container()
                    .with(Bounds::fill_all())
                    .with(Appearance::Background(CONTROL_BACKGROUND))
                    .with(Font::Size(13.0))
                    .with(Font::Weight(FontWeight::Normal))
                    .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Never))
                    .with(Scroll::VerticalScrollBar(ScrollBarVisibility::OnlyIfNeeded))
                    .with(effect_controls)
            ])
    });

    // Create a controller to add the effects when necessary (just waiting for the click events, which have the effect ID as a parameter)
    let model                           = model.clone();
    let available_effects               = anim_model.available_effects.clone();
    let add_popup_open                  = anim_model.add_popup_open.clone();
    let selected_animation_elements     = anim_model.selected_animation_elements.clone();

    let effect_controller               = ImmediateController::with_ui(new_effects_ui,
        move |events, _actions, _resources| {
            // Set up for running the event loop
            let mut events                      = events;
            let model                           = model.clone();
            let add_popup_open                  = add_popup_open.clone();
            let available_effects               = available_effects.clone();
            let selected_animation_elements     = selected_animation_elements.clone();

            // Run the event loop and wait until we get a click on a known event
            async move {
                while let Some(event) = events.next().await {
                    match event {
                        ControllerEvent::Action(name, _) => {
                            // The event name should match the effect ID
                            let available_effects   = available_effects.get();
                            let clicked_effect      = available_effects.iter().filter(|effect| effect.id.as_str() == name.as_str()).nth(0);

                            if let Some(clicked_effect) = clicked_effect {
                                // Add the effect to the animation
                                let selected_elements = selected_animation_elements.get().into_iter().map(|elem| elem.id()).collect::<Vec<_>>();
                                model.edit().publish(Arc::new(clicked_effect.create.iter()
                                    .map(|edit| AnimationEdit::Element(selected_elements.clone(), edit.clone()))
                                    .collect())).await;
                                
                                // Close the popup
                                add_popup_open.set(false);
                            }
                        }
                    }
                }
            }
        });

    // Wrap in a popup controller
    PopupController::new(effect_controller, &anim_model.add_popup_open)
        .with_direction(&PopupDirection::Below)
        .with_size(&(200, 250))
}

///
/// Creates the binding for the animation sidebar user interface
///
fn animation_sidebar_ui<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, anim_model: Arc<AnimationControllerModel>) -> BindRef<Control> {
    // Create a binding for the base animation type for the selected element
    let edit_counter                    = model.frame_update_count();
    let selected_animation_elements_2   = anim_model.selected_animation_elements.clone();
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
            let element_animation_type  = selected_element.description().effect().base_animation_type();

            element_animation_type
        }
    });

    let model = Arc::clone(model);

    // Generate the UI for the animation panel
    computed(move || {
        let selected_elements = anim_model.selected_animation_elements.get();

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

            // The effect selector lets the user pick between the list of sub-effects available
            let effect_height       = 20.0;
            let selected_effect     = model.selection().selected_sub_effect.get();
            let available_effects   = anim_model.effects.get();

            let effect_controls     = available_effects.into_iter()
                .enumerate()
                .flat_map(|(idx, (_elem_id, effect))| {
                    let is_selected = Some(effect.address()) == selected_effect.as_ref().map(|(_, effect)| effect.address());
                    let background  = if (idx%2) == 0 { CONTROL_BACKGROUND } else { CONTROL_BACKGROUND_2 };
                    let background  = if is_selected { SELECTED_BACKGROUND } else { background };

                    let title       = effect.effect_type().description().to_string();
                    let action_name = format!("SelectEffect {}", idx);

                    vec![
                        Control::container()
                            .with(Bounds::next_vert(effect_height))
                            .with(Appearance::Background(background))
                            .with(ControlAttribute::Padding((8, 0), (8, 0)))
                            .with((ActionTrigger::Click, action_name.clone()))
                            .with(vec![
                                Control::label().with(Bounds::fill_all()).with((ActionTrigger::Click, action_name)).with(title)
                            ]),
                        Control::empty()
                            .with(Bounds::next_vert(1.0))
                            .with(Appearance::Background(CONTROL_BORDER)),
                    ]
                })
                .collect::<Vec<_>>();

            let effect_selector = Control::container()
                .with(Bounds::stretch_vert(1.0))
                .with(Font::Size(11.0))
                .with(ControlAttribute::Padding((1, 1), (1, 1)))
                .with(Appearance::Background(CONTROL_BORDER))
                .with(vec![
                    Control::scrolling_container()
                        .with(Bounds::fill_all())
                        .with(Appearance::Background(CONTROL_BACKGROUND))
                        .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Never))
                        .with(Scroll::VerticalScrollBar(ScrollBarVisibility::Always))
                        .with(effect_controls)
                ]);

            // The action buttons for adding new effects
            let action_buttons = Control::container()
                .with(Bounds::next_vert(22.0))
                .with(Font::Size(9.0))
                .with(vec![
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0)),
                    Control::container()
                        .with(Hint::Class("button-group".to_string()))
                        .with(Bounds::next_horiz(22.0*4.0 + 4.0))
                        .with(Font::Size(13.0))
                        .with(Font::Weight(FontWeight::ExtraBold))
                        .with(vec![
                            Control::button()
                                .with((ActionTrigger::Click, "OpenNewEffectPopup"))
                                .with(vec![
                                    Control::label()
                                        .with(Bounds::fill_all())
                                        .with(TextAlign::Center)
                                        .with("+")
                                        .with_controller("NewEffect")
                                ])
                                .with(Bounds::next_horiz(24.0)),
                            Control::button()
                                .with(vec![
                                    Control::label()
                                        .with(Bounds::fill_all())
                                        .with(TextAlign::Center)
                                        .with("-")
                                ])
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(Bounds::next_horiz(24.0)),
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
                        .with(Bounds::next_vert(2.0)),

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
                    // These are the set of actions for selecting the base animation type
                    let base_animation_type = action_name["SetBase ".len()..].to_string();
                    let base_animation_type = BaseAnimationType::from_str(&base_animation_type);

                    // Set base animation type action
                    if let Ok(base_animation_type) = base_animation_type {
                        AnimationAction::SetBaseAnimationType(base_animation_type)
                    } else {
                        AnimationAction::Unknown(action_name.clone())
                    }

                } else if action_name.starts_with("SelectEffect ") {
                    // These are the set of actions for selecting one of the effects for the current animation selection
                    let effect_index = action_name["SelectEffect ".len()..].to_string();
                    let effect_index = usize::from_str(&effect_index);

                    effect_index.map(|idx| AnimationAction::SelectEffect(idx)).unwrap_or_else(|_| AnimationAction::Unknown(action_name.clone()))
                } else {
                    // Match against single action names
                    match action_name.as_str() {
                        "OpenNewEffectPopup" => AnimationAction::OpenNewEffectPopup,

                        // Action name doesn't match any we know
                        _ => AnimationAction::Unknown(action_name.clone())
                    }
                }
            }
        }
    }
}

fn refresh_selected_sub_effect<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, anim_model: &Arc<AnimationControllerModel>) {
    // Fetch the selected animation elements and the currently selected sub-effect (there's nothing to do if no sub-effect is selected)
    let (subeffect_id, subeffect_description)   = if let Some(sub_effect) = model.selection().selected_sub_effect.get() { sub_effect } else { return; };
    let selected_animation_elements             = anim_model.selected_animation_elements.get();

    if selected_animation_elements.len() != 1 {
        // Currently can only pick a sub-effect if there's only one element selected
        model.selection().selected_sub_effect.set(None);
    } else if !selected_animation_elements.iter().any(|elem| elem.id() == subeffect_id) {
        // Animation is no longer in the selected list
        model.selection().selected_sub_effect.set(None);
    } else {
        // Might need to update to the latest description for this sub-effect (or clear it if it's no longer present with the same address)
        let animation_element   = &selected_animation_elements[0];
        let active_effects      = animation_element.effect().sub_effects();
        let expected_address    = subeffect_description.address();
        let selected_effect     = active_effects.into_iter().filter(|effect| &effect.address() == &expected_address).nth(0);
        let selected_effect     = selected_effect.map(|effect| Arc::new(effect));

        if let Some(selected_effect) = selected_effect {
            if selected_effect.effect_type() != subeffect_description.effect_type() {
                // The effect type has changed, so the selection is pointing at a different effect
                model.selection().selected_sub_effect.set(None);
            } else if selected_effect != subeffect_description {
                // Update the selection if the effect description has changed
                model.selection().selected_sub_effect.set(Some((subeffect_id, selected_effect)));
            }
        } else {
            // No effect with this address is presen
            model.selection().selected_sub_effect.set(None);
        }
    }
}

///
/// Creates a binding of the list of effects for the currently selected animation element
///
fn effects_list(selected_animation_elements: &BindRef<Vec<SelectedAnimationElement>>) -> BindRef<Vec<(ElementId, Arc<SubEffectDescription>)>> {
    let selected_animation_elements = selected_animation_elements.clone();

    computed(move || {
        let selected_animation_elements = selected_animation_elements.get();

        if selected_animation_elements.len() == 1 {
            // When there's only one element selected, we generate a list of sub-effects that the user can choose between
            let element_id  = selected_animation_elements[0].id();
            let sub_effects = selected_animation_elements[0].effect().sub_effects();

            sub_effects.into_iter()
                .map(|effect| (element_id, Arc::new(effect)))
                .collect()
        } else {
            // For 0 or >1 element, there are no subeffects available
            vec![]
        }
    })
    .into()
}

///
/// Runs the animation sidebar panel
///
async fn run_animation_sidebar_panel<Anim: 'static+EditableAnimation>(events: ControllerEventStream, actions: mpsc::Sender<ControllerAction>, _resources: ControllerResources, model: Arc<FloModel<Anim>>, anim_model: Arc<AnimationControllerModel>) {
    let mut actions         = actions;

    // Setup: set 'no selected subeffect' when the runtime starts (in case it has some stale value in it)
    model.selection().selected_sub_effect.set(None);

    // Create sub-controllers and initialise the UI
    let new_effect_ctrl     = new_effect_controller(&model, &anim_model);
    let ui                  = animation_sidebar_ui(&model, anim_model.clone());

    actions.send(ControllerAction::AddSubController("NewEffect".to_string(), Arc::new(new_effect_ctrl))).await.ok();
    actions.send(ControllerAction::SetUi(ui.into())).await.ok();

    // Generate some events from the model: we need to know when the canvas is updated or the selection is changed to update the selected sub-effect
    let canvas_updated      = follow(model.frame_update_count().clone()).map(|_| AnimationAction::CanvasUpdated);
    let selection_changed   = follow(anim_model.selected_animation_elements.clone()).map(|_| AnimationAction::SelectionChanged);

    // Convert the events into animation events
    let events              = events.map(|event| AnimationAction::from_controller_event(&event));

    // Combine to create the complete list of events
    let mut events          = stream::select_all(vec![events.boxed(), canvas_updated.boxed(), selection_changed.boxed()]);

    // Run while there are events pending
    while let Some(event) = events.next().await {
        match event {
            // Unknown events are logged
            AnimationAction::Unknown(evt)   => { warn!("Unknown animation sidebar event {}", evt); }

            AnimationAction::SetBaseAnimationType(new_base_type) => {
                // Set the base animation type for the selected region
                let element_ids = anim_model.selected_animation_elements.get().iter().map(|elem| elem.id()).collect();
                let edits       = vec![AnimationEdit::Element(element_ids, ElementEdit::SetAnimationBaseType(new_base_type))];

                // Send the edits to the animation and wait for them to be received
                let mut editor  = model.edit();
                editor.publish(Arc::new(edits)).await;
                editor.when_empty().await;
            }

            AnimationAction::SelectEffect(effect_idx) => {
                // Change the selected effect in the main animation
                let effects         = anim_model.effects.get();
                let selected_effect = if effect_idx < effects.len() {
                    Some(effects[effect_idx].clone())
                } else {
                    None
                };

                model.selection().selected_sub_effect.set(selected_effect);
            }

            AnimationAction::OpenNewEffectPopup => {
                // This just opens the 'add effect' popup
                anim_model.add_popup_open.set(true);
            }

            AnimationAction::SelectionChanged |
            AnimationAction::CanvasUpdated => {
                // The selected sub-effect may no longer be available or may have an out-of-date description
                refresh_selected_sub_effect(&model, &anim_model);
            },
        }
    }
}

///
/// The Animation panel is used to show an overview of the effects in the currently selected animation element(s)
///
pub fn animation_sidebar_panel<Anim: 'static+EditableAnimation>(model: &Arc<FloModel<Anim>>, selected_animation_elements: BindRef<Vec<SelectedAnimationElement>>) -> SidebarPanel {
    // Create the controller for the panel
    let effects             = effects_list(&selected_animation_elements);
    let animation_model     = Arc::new(AnimationControllerModel {
        selected_animation_elements:    selected_animation_elements,
        effects:                        effects,
        add_popup_open:                 bind(false),
        available_effects:              bind(default_effects()).into(),
    });

    let model               = model.clone();
    let anim_model_clone    = animation_model.clone();
    let controller          = ImmediateController::empty(move |events, actions, resources| 
        run_animation_sidebar_panel(events, actions, resources, model.clone(), anim_model_clone.clone()));

    // The panel is 'active' if there is one or more elements selected
    let is_active           = computed(move || animation_model.selected_animation_elements.get().len() > 0);

    // Construct the sidebar panel
    SidebarPanel::with_title("Animation")
        .with_active(BindRef::from(is_active))
        .with_height(bind(200.0))
        .with_controller(controller)
}
