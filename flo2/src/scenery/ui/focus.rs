use super::control_id::*;
use super::focus_events::*;
use super::subprograms::*;
use super::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::*;
use flo_curves::geo::*;
use flo_curves::bezier::path::*;
use flo_curves::bezier::rasterize::*;
use flo_curves::bezier::vectorize::*;

use futures::prelude::*;
use serde::*;

use std::collections::{HashMap, HashSet};

///
/// Requests to the focus subprogram.
///
/// The focus subprogram deals with mapping mouse clicks on a document window to the subprogram responsible for
/// processing them, as well as routing keyboard events to the control that currently has focus.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Focus {
    /// An event in need of routing
    Event(DrawEvent),

    /// An update from the scene (used to track when subprograms go away)
    Update(SceneUpdate),

    /// Sets the subprogram that should process keyboard events
    SetKeyboardFocus(SubProgramId, ControlId),

    /// Sets which control should receive keyboard focus after the specified control (within a subprogram, which might have several controls)
    ///
    /// This moves the first control so that it's ordered before the second control (new controls are added at the end of the list)
    SetFollowingControl(SubProgramId, ControlId, ControlId),

    /// Sets which subprogram should receive keyboard focus after reaching the end of the controls in the first subprogram 
    ///
    /// This moves the first subprogram so that it's ordered before the second program (new subprograms are added at the end of the list)
    SetFollowingSubProgram(SubProgramId, SubProgramId),

    /// Move keyboard focus to the next control
    FocusNext,

    /// Move keyboard focus to the preceding control
    FocusPrevious,

    /// Sets which subprogram receives canvas events (events that don't hit any control region)
    SetCanvas(SubProgramId),

    /// Claims a region inside the specified path as belonging to the specified subprogram. The z-index is used to disambiguate requests if more than region matches
    /// Clicks in this region will have 'None' as the control ID
    ClaimRegion { program: SubProgramId, region: Vec<UiPath>, z_index: usize },

    /// Claims a region for a single control within the region for a subprogram. The z-index here is used to disambiguate when multiple regions matches
    ClaimControlRegion { program: SubProgramId, region: Vec<UiPath>, control: ControlId, z_index: usize },

    /// Removes a claim added by ClaimRegion
    RemoveClaim(SubProgramId),

    /// Removes a claim added by ClaimControlRegion
    RemoveControlClaim(SubProgramId, ControlId),
}

///
/// The current state of the pointer buttons
///
struct ButtonState {
    /// Set of the buttons that are currently down
    is_down: HashSet<Button>,
}

impl ButtonState {
    ///
    /// Creates a new button state
    ///
    pub fn new() -> Self {
        ButtonState {
            is_down: HashSet::new()
        }
    }

    ///
    /// Updates the buttons that are down
    ///
    pub fn set_buttons<'a>(&mut self, pressed_buttons: impl IntoIterator<Item=&'a Button>) {
        self.is_down = pressed_buttons.into_iter().copied().collect();
    }

    ///
    /// Number of buttons that are down
    ///
    pub fn num_buttons_down(&self) -> usize {
        self.is_down.len()
    }
}


///
/// Runs the UI focus subprogram
///
pub async fn focus(input: InputStream<Focus>, context: SceneContext) {
    let program_id  = context.current_program_id().unwrap();
    let mut input   = input;

    // Request updates from the scene (which we'll use to remove subprograms that aren't running any more)
    context.send_message(SceneControl::Subscribe(program_id.into())).await.ok();

    // Create the state
    let mut focus = FocusProgram {
        canvas_program:             None,
        subprogram_space:           None,
        subprogram_data:            HashMap::new(),
        subprogram_order:           vec![],
        pointer_target:             None,
        pointer_target_program:     None,
        pointer_target_control:     None,
        button_state:               ButtonState::new(),
        focused_subprogram:         None,
        focused_control:            None,
        focused_event_target:       None,
        tab_ordering:               HashMap::new(),
        scale:                      None,
        bounds:                     None,
        hover:                      (None, None),
    };

    while let Some(request) = input.next().await {
        use Focus::*;

        match request {
            // General untargeted draw events
            Event(DrawEvent::Redraw)                => { },
            Event(DrawEvent::NewFrame)              => { },
            Event(DrawEvent::Scale(scale))          => { focus.set_scale(scale, &context).await; },
            Event(DrawEvent::Resize(w, h))          => { focus.set_bounds(w, h, &context).await; },
            Event(DrawEvent::CanvasTransform(_))    => { },
            Event(DrawEvent::Closed)                => { focus.send_to_all(FocusWindowEvent::Closed, &context).await; }

            // Pointer and key events
            Event(DrawEvent::Pointer(PointerAction::Enter, _, _))                           => { },
            Event(DrawEvent::Pointer(PointerAction::Leave, _, _))                           => { },
            Event(DrawEvent::Pointer(PointerAction::ButtonDown, pointer_id, pointer_state)) => { focus.set_pointer_target(&pointer_state, &context).await; focus.button_state.set_buttons(&pointer_state.buttons); focus.send_to_pointer_target(FocusPointerEvent::Pointer(None, PointerAction::ButtonDown, pointer_id, pointer_state)).await; },
            Event(DrawEvent::Pointer(PointerAction::ButtonUp, pointer_id, pointer_state))   => { focus.button_state.set_buttons(&pointer_state.buttons); focus.send_to_pointer_target(FocusPointerEvent::Pointer(None, PointerAction::ButtonUp, pointer_id, pointer_state)).await; },
            Event(DrawEvent::Pointer(other_action, pointer_id, pointer_state))              => { focus.set_pointer_target(&pointer_state, &context).await; focus.send_to_pointer_target(FocusPointerEvent::Pointer(None, other_action, pointer_id, pointer_state)).await; },
            Event(DrawEvent::KeyDown(scancode, key))                                        => { focus.send_to_focus(FocusKeyboardEvent::KeyDown(None, scancode, key), &context).await; },
            Event(DrawEvent::KeyUp(scancode, key))                                          => { focus.send_to_focus(FocusKeyboardEvent::KeyUp(None, scancode, key), &context).await; },

            // Updates from the scene in general
            Update(SceneUpdate::Stopped(program_id))    => { focus.remove_program_claims(program_id).await; focus.remove_program_focus(program_id).await; },
            Update(_)                                   => { }

            // Keyboard handling
            SetKeyboardFocus(program_id, control_id)                        => focus.set_keyboard_focus(program_id, control_id, &context).await,
            SetFollowingControl(program_id, control_id, next_control_id)    => focus.set_following_control(program_id, control_id, next_control_id).await,
            SetFollowingSubProgram(program_id, next_program_id)             => focus.set_following_subprogram(program_id, next_program_id).await,
            FocusNext                                                       => focus.focus_next(&context).await,
            FocusPrevious                                                   => focus.focus_previous(&context).await,

            // Control handling
            RemoveClaim(program_id)                                     => focus.remove_program_claims(program_id).await,
            RemoveControlClaim(program_id, control_id)                  => focus.remove_control_claims(program_id, control_id).await,
            SetCanvas(canvas_program_id)                                => focus.set_canvas(canvas_program_id).await,
            ClaimRegion { program, region, z_index }                    => focus.claim_region(program, region, None, z_index, &context).await,
            ClaimControlRegion { program, region, control, z_index }    => focus.claim_region(program, region, Some(control), z_index, &context).await,
        }
    }
}

impl SceneMessage for Focus {
    fn default_target() -> StreamTarget {
        subprogram_focus().into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Set up filters for the focus events/updates
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|scene_updates| scene_updates.map(|update| Focus::Update(update)))), (), StreamId::with_message_type::<SceneUpdate>()).ok();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|draw_events| draw_events.map(|event| Focus::Event(event)))), (), StreamId::with_message_type::<DrawEvent>()).ok();

        // Create the standard focus subprogram when a message is sent for the first tiem
        init_context.add_subprogram(subprogram_focus(), focus, 20);

        // This is the default target for focus messages to this scene
        init_context.connect_programs((), subprogram_focus(), StreamId::with_message_type::<Focus>()).ok();
    }
}

///
/// Represents a control within a subprogram
///
struct SubProgramControl {
    id:         ControlId,
    bounds:     Bounds<UiPoint>,
    region:     PathContour,
    z_index:    usize,
}

///
/// Definition for a region of the canvas where a subprogram owns the events
///
struct SubProgramRegion {
    region:     PathContour,
    bounds:     Bounds<UiPoint>,
    controls:   Vec<SubProgramControl>,
    z_index:    usize,
}

///
/// Represents the ordering of subprograms that can receive keyboard focus
///
struct KeyboardSubProgram {
    control_order:  Vec<ControlId>,
}

///
/// Helps manage the state for the focus subprogram
///
struct FocusProgram {
    /// The program that canvas events get sent to (events that aren't for any region)
    canvas_program: Option<SubProgramId>,

    /// x-oriented 1D scan space for subprogram regions (or None if this hasn't been calculated)
    subprogram_space: Option<Space1D<SubProgramId>>,

    /// The data for each subprogram region
    subprogram_data: HashMap<SubProgramId, SubProgramRegion>,

    /// The control that pointer events should be sent to
    pointer_target: Option<OutputSink<FocusPointerEvent>>,

    /// The subprogram ID of the active pointer target
    pointer_target_program: Option<SubProgramId>,

    /// The control of the active pointer target
    pointer_target_control: Option<ControlId>,

    /// State of the mouse buttons
    button_state: ButtonState,

    /// The subprogram that currently has keyboard focus
    focused_subprogram: Option<SubProgramId>,

    /// The control within the subprogram that has keyboard focus
    focused_control: Option<ControlId>,

    /// Where keyboard events should be sent
    focused_event_target: Option<OutputSink<FocusKeyboardEvent>>,

    /// The tab ordering for the controls within this program
    tab_ordering: HashMap<SubProgramId, KeyboardSubProgram>,

    /// The focus order for the subprograms
    subprogram_order: Vec<SubProgramId>,

    /// The bounding box of the window (None if this has not been sent to us)
    bounds: Option<(f64, f64)>,

    /// The scale of the window (None if this has not been sent to us)
    scale: Option<f64>,

    /// The last control that we found we were hovering over
    hover: (Option<SubProgramId>, Option<ControlId>),
}

impl SubProgramRegion {
    ///
    /// Returns true if the specified point is inside this region
    ///
    fn point_is_inside(&self, x: f64, y: f64) -> bool {
        // Note that PathContour doesn't support negative values for x

        let intercepts = self.region.intercepts_on_line(y);
        if intercepts.into_iter().any(|intercept| intercept.contains(&x)) {
            // Intercept in the region for this program
            true
        } else {
            // Could be an intercept on any of the controls in this region
            self.controls.iter()
                .filter(|control| UiPoint(x, y).in_bounds(&control.bounds))
                .any(|control| control.point_is_inside(x, y))
        }
    }
}

impl SubProgramControl {
    ///
    /// Returns true if the specified point is inside this region
    ///
    fn point_is_inside(&self, x: f64, y: f64) -> bool {
        // Note that PathContour doesn't support negative values for x

        let intercepts = self.region.intercepts_on_line(y);
        intercepts.into_iter().any(|intercept| intercept.contains(&x))
    }
}

impl FocusProgram {
    ///
    /// Sets the scale of the window
    ///
    async fn set_scale(&mut self, scale: f64, context: &SceneContext) {
        self.scale = Some(scale);
        self.send_to_all(FocusWindowEvent::Scale(scale), &context).await;
    }

    ///
    /// Sets the bounds of the window
    ///
    async fn set_bounds(&mut self, width: f64, height: f64, context: &SceneContext) {
        self.bounds = Some((width, height));
        self.send_to_all(FocusWindowEvent::Resize(width, height), &context).await;
    }

    ///
    /// Sends greeting messages to a newly added subprogram
    ///
    async fn greet_new_subprogram(&mut self, subprogram_id: SubProgramId, context: &SceneContext) {
        let subprogram = context.send(subprogram_id).ok();

        if let Some(mut subprogram) = subprogram {
            // Send the scale if it's been stored
            if let Some(scale) = self.scale {
                subprogram.send(FocusWindowEvent::Scale(scale)).await.ok();
            }

            // Send the bounds if they've been stored
            if let Some((w, h)) = self.bounds {
                subprogram.send(FocusWindowEvent::Resize(w, h)).await.ok();
            }
        }
    }

    ///
    /// Sets keyboard focus to a specific program/control
    ///
    async fn set_keyboard_focus(&mut self, program_id: SubProgramId, control_id: ControlId, context: &SceneContext) {
        // Unfocus the existing program
        if let (Some(old_program_id), Some(old_control_id)) = (self.focused_subprogram, self.focused_control) {
            if let Ok(mut channel) = context.send(old_program_id) {
                channel.send(FocusKeyboardEvent::Unfocused(old_control_id)).await.ok();
            }

            self.focused_subprogram  = None;
            self.focused_control     = None;
        }

        // Update the focused control and inform the relevant program
        if let Ok(mut channel) = context.send(program_id) {
            self.focused_subprogram  = Some(program_id);
            self.focused_control     = Some(control_id);
            channel.send(FocusKeyboardEvent::Focused(control_id)).await.ok();

            self.focused_event_target = Some(channel);
        } else {
            self.focused_event_target = None;
        }
    }

    ///
    /// Sets the following control for keyboard focus (inserting control_id before next_control_id)
    ///
    async fn set_following_control(&mut self, program_id: SubProgramId, control_id: ControlId, next_control_id: ControlId) {
        // Find the subprogram for these controls
        if !self.subprogram_order.iter().any(|prog| prog == &program_id) {
            self.subprogram_order.push(program_id);
        }

        let controls_for_program = self.tab_ordering.entry(program_id)
            .or_insert_with(|| KeyboardSubProgram {
                control_order: vec![],
            });

        // Remove the control if it already has an order
        controls_for_program.control_order.retain(|ctrl| ctrl != &control_id);

        // Add the first control to the end of the list for the program if it's not there already (generally this should be called with controls that already exist)
        let before_idx = if let Some(idx) = controls_for_program.control_order.iter().position(|ctrl| ctrl == &next_control_id) {
            idx
        } else {
            let idx = controls_for_program.control_order.len();
            controls_for_program.control_order.push(next_control_id);

            idx
        };

        // Insert the control before the 'next' control
        controls_for_program.control_order.insert(before_idx, control_id);
    }

    ///
    /// Sets the following subprogram for keyboard focus (inserting program_id before next_program_id)
    ///
    async fn set_following_subprogram(&mut self, program_id: SubProgramId, next_program_id: SubProgramId) {
        // Ensure that the control order exists for the programs
        self.tab_ordering.entry(program_id)
            .or_insert_with(|| KeyboardSubProgram {
                control_order: vec![],
            });
        self.tab_ordering.entry(next_program_id)
            .or_insert_with(|| KeyboardSubProgram {
                control_order: vec![],
            });

        // Remove program_id from the existing list
        self.subprogram_order.retain(|prog| prog != &program_id);
        
        // Find the index to add the program ID before
        let before_idx = if let Some(idx) = self.subprogram_order.iter().position(|prog| prog == &next_program_id) {
            idx
        } else {
            // Add next_program_id at the end of the ordering if it doesn't already exist
            let idx = self.subprogram_order.len();
            self.subprogram_order.push(next_program_id);

            idx
        };

        // Insert program_id before next_program_id
        self.subprogram_order.insert(before_idx, program_id);
    }

    ///
    /// Figures out the following subprogram ID in focus order
    ///
    fn next_subprogram(&self, current_program: Option<SubProgramId>) -> Option<SubProgramId> {
        let current_program = current_program?;
        let current_idx     = self.subprogram_order.iter().position(|prog| prog == &current_program)?;
        let next_idx        = if current_idx+1 >= self.subprogram_order.len() { 0 } else { current_idx+1 };

        Some(self.subprogram_order[next_idx])
    }

    ///
    /// Figures out the following control
    ///
    fn next_control(&self, current_program: Option<SubProgramId>, current_control: Option<ControlId>) -> Option<(SubProgramId, ControlId)> {
        let current_program = current_program?;
        let current_control = current_control?;

        // Get the data for the current control
        let program_data    = self.tab_ordering.get(&current_program)?;
        let control_pos     = program_data.control_order.iter().position(|ctrl| ctrl == &current_control)?;

        // If this would loop back to the beginning then focus the next subprogram
        if control_pos+1 >= program_data.control_order.len() {
            let next_program_id = self.next_subprogram(Some(current_program))?;
            let next_program    = self.tab_ordering.get(&next_program_id)?;
            let first_control   = next_program.control_order.iter().copied().next()?;

            Some((next_program_id, first_control))
        } else {
            // Just the next control in the same program
            Some((current_program, program_data.control_order[control_pos+1]))
        }
    }

    ///
    /// Focuses the next control in the list
    ///
    async fn focus_next(&mut self, context: &SceneContext) {
        if let Some((next_program, next_control)) = self.next_control(self.focused_subprogram, self.focused_control) {
            // Currently focused control has a 'next'
            self.set_keyboard_focus(next_program, next_control, context).await;
        } else {
            // TODO: focus the first control in the first program
        }
    }

    ///
    /// Figures out the previous subprogram ID in focus order
    ///
    fn previous_subprogram(&self, current_program: Option<SubProgramId>) -> Option<SubProgramId> {
        let current_program = current_program?;
        let current_idx     = self.subprogram_order.iter().position(|prog| prog == &current_program)?;
        let previous_idx    = if current_idx == 0 { self.subprogram_order.len()-1 } else { current_idx-1 };

        Some(self.subprogram_order[previous_idx])
    }

    ///
    /// Figures out the preceding control
    ///
    fn previous_control(&self, current_program: Option<SubProgramId>, current_control: Option<ControlId>) -> Option<(SubProgramId, ControlId)> {
        let current_program = current_program?;
        let current_control = current_control?;

        // Get the data for the current control
        let program_data    = self.tab_ordering.get(&current_program)?;
        let control_pos     = program_data.control_order.iter().position(|ctrl| ctrl == &current_control)?;

        if control_pos == 0 {
            // Return the last control of the previous program
            let previous_program_id = self.previous_subprogram(Some(current_program))?;
            let previous_program    = self.tab_ordering.get(&previous_program_id)?;
            let previous_control    = previous_program.control_order.last()?;

            Some((previous_program_id, *previous_control))
        } else {
            // Retur nthe preceding control
            Some((current_program, program_data.control_order[control_pos-1]))
        }
    }

    ///
    /// Focuses the previous control in the list
    ///
    async fn focus_previous(&mut self, context: &SceneContext) {
        if let Some((previous_program, previous_control)) = self.previous_control(self.focused_subprogram, self.focused_control) {
            // Currently focused control has a 'next'
            self.set_keyboard_focus(previous_program, previous_control, context).await;
        } else {
            // TODO: focus the last control in the last program
        }
    }

    ///
    /// Sets the program where mouse events go if there's no region defined
    ///
    async fn set_canvas(&mut self, canvas_program: SubProgramId) {
        self.canvas_program = Some(canvas_program);
    }

    ///
    /// Marks a region as belonging to a certain control
    ///
    async fn claim_region(&mut self, program: SubProgramId, region: Vec<UiPath>, control: Option<ControlId>, z_index: usize, context: &SceneContext) {
        // Get the bounds of the region
        let bounds = region.iter()
            .map(|path| path.bounding_box::<Bounds<_>>())
            .reduce(|a, b| a.union_bounds(b))
            .unwrap_or(Bounds::empty());

        // Create a path contour from the region (we use a contour size of 0, 0 as we're not actually scan-converting this path)
        let contour_size    = ContourSize(bounds.max().x().max(0.0) as _, bounds.max().y().max(0.0) as _);
        let region          = PathContour::from_path(region, contour_size);

        // Look up or create the subprogram data
        let program_data = self.subprogram_data.get_mut(&program);
        let program_data = if let Some(program_data) = program_data {
            program_data
        } else {
            // Add a new region
            let new_region = SubProgramRegion {
                region:     PathContour::from_path::<UiPath>(vec![], contour_size),
                bounds:     bounds.clone(),
                controls:   vec![],
                z_index:    0,
            };
            self.subprogram_data.insert(program, new_region);

            // Send the greeting to the subprogram
            self.greet_new_subprogram(program, context).await;

            // Fetch the program we just added
            self.subprogram_data.get_mut(&program).unwrap()
        };

        if let Some(control) = control {
            // Make sure the bounds includes the region
            program_data.bounds = program_data.bounds.union_bounds(bounds);

            // Add a new control
            program_data.controls.push(SubProgramControl {
                id:         control,
                region:     region,
                bounds:     bounds,
                z_index:    z_index,
            })
        } else {
            // Not setting the region for a control: update the region set for the subprogram
            let combined_bounds = program_data.controls.iter()
                .fold(bounds, |a, b| a.union_bounds(b.bounds));

            program_data.region   = region;
            program_data.bounds   = combined_bounds;
            program_data.z_index  = z_index;
        }

        // Space becomes None (need to recalculate it before we can handle click events)
        self.subprogram_space = None;
    }

    ///
    /// Removes all claims to space that match the specified subprogram
    ///
    async fn remove_program_claims(&mut self, program: SubProgramId) {
        self.subprogram_data.remove(&program);
        self.subprogram_space = None;
    }

    ///
    /// Removes all claims to space that match the specified subprogram
    ///
    async fn remove_program_focus(&mut self, program: SubProgramId) {
        self.tab_ordering.remove(&program);
        self.subprogram_order.retain(|prog| prog != &program);
    }

    ///
    /// Removes the claim that matches the specified control
    ///
    async fn remove_control_claims(&mut self, program: SubProgramId, control: ControlId) {
        if let Some(program_data) = self.subprogram_data.get_mut(&program) {
            program_data.controls.retain(|item| item.id != control);
        }
    }

    ///
    /// Sends an event to whichever program/control is focused
    ///
    async fn send_to_focus(&mut self, event: FocusKeyboardEvent, context: &SceneContext) {
        let control = self.focused_control;
        let event   = event.with_target(control);

        if let Some(pointer_target_program) = &mut self.pointer_target_program {
            // While performing actions, we try to send to the pointer target instead (if it accepts keyboard events)
            if let Ok(mut pointer_target) = context.send(*pointer_target_program) {
                pointer_target.send(event).await.ok();
            }
        } else if let Some(focus_target) = &mut self.focused_event_target {
            // Send to the keyboard focus if there's no pointer target
            if focus_target.send(event).await.is_err() {
                self.focused_event_target = None;
            }
        }
    }

    ///
    /// Determines the target program at a location in the canvas, filtering out any programs/controls that shouldn't be matched
    ///
    fn pointer_target_filter(&mut self, location_in_canvas: Option<(f64, f64)>, should_match: impl Fn(SubProgramId, Option<ControlId>) -> bool) -> (Option<SubProgramId>, Option<ControlId>) {
        let space                   = &mut self.subprogram_space;
        let subprogram_data         = &self.subprogram_data;

        // Generate the space if it's not already generated
        let space = if let Some(space) = space {
            space
        } else {
            *space = Some(Space1D::from_data(subprogram_data.iter().map(|(program_id, region)| (region.bounds.min().x()..region.bounds.max().x(), *program_id))));
            space.as_mut().unwrap()
        };

        // Locate the subprogram that the pointer is over
        if let Some((x, y)) = location_in_canvas {
            // Find all of the subprograms where the point might be inside
            let mut possible_target_programs = space.data_at_point(x)
                .flat_map(|subprogram_id| subprogram_data.get(subprogram_id).map(|region| (subprogram_id, region)))
                .filter(|(_, region)| UiPoint(x, y).in_bounds(&region.bounds))
                .filter(|(_, region)| region.point_is_inside(x, y))
                .collect::<Vec<_>>();

            // Order by z-index if there are multiple possibilities
            if possible_target_programs.len() > 1 {
                possible_target_programs.sort_by_key(|(_, region)| region.z_index);
            }

            // Locate the control in the target program
            let possible_controls = possible_target_programs.into_iter()
                .rev()                          // Because the 'nearest' program is last due to the ordering
                .map(|(program_id, _)| program_id)
                .flat_map(|target_program| {
                    let target_program      = *target_program;
                    let target_program_data = subprogram_data.get(&target_program);

                    target_program_data
                        .into_iter()
                        .flat_map(move |target_program_data| {
                            // Find the control that the point might be inside
                            let mut possible_controls = target_program_data.controls.iter()
                                .filter(|control| UiPoint(x, y).in_bounds(&control.bounds))
                                .filter(|control| control.point_is_inside(x, y))
                                .collect::<Vec<_>>();

                            // Order by z-index if there are multiple possibilities
                            if possible_controls.len() > 1 {
                                possible_controls.sort_by_key(|control| control.z_index);
                            }

                            // If no controls match, then add in the 'base' program canvas
                            let base = if possible_controls.is_empty() {
                                Some((target_program, None))
                            } else {
                                None
                            };

                            // The highest z-index is the target control
                            possible_controls.into_iter()
                                .rev()
                                .map(move |control| (target_program, Some(control.id)))
                                .chain(base)
                        })
                });

            // Apply the filter to the results (which are now a set of possibilities in descending order)
            let mut filtered_controls = possible_controls.filter(|(program_id, control_id)| should_match(*program_id, *control_id));

            // The first item in this iterator is the 'topmost' control that isn't filtered
            if let Some((program_id, control_id)) = filtered_controls.next() {
                (Some(program_id), control_id)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    }

    ///
    /// Determines the target program at a location in the canvas
    ///
    #[inline]
    fn pointer_target(&mut self, location_in_canvas: Option<(f64, f64)>) -> (Option<SubProgramId>, Option<ControlId>) {
        self.pointer_target_filter(location_in_canvas, |_, _| true)
    }

    ///
    /// Send a hover message to the pointer target
    ///
    async fn send_hover(&mut self, pointer_state: &PointerState) {
        // 'Hovering' only happens when a mouse button is held down
        if self.button_state.num_buttons_down() == 0 {
            self.hover = (None, None);
            return;
        }

        // Locate the subprogram that the pointer is over
        let current_target          = self.pointer_target_program;
        let current_target_control  = self.pointer_target_control;

        let (hover_program, hover_control) = self.pointer_target_filter(pointer_state.location_in_canvas, |program_id, control_id| {
            if Some(program_id) == current_target {
                if control_id.is_none() || control_id == current_target_control {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        if &self.hover != &(hover_program, hover_control) {
            // Replace the hover control
            self.hover = (hover_program, hover_control);

            // Send a hover event
            if let Some(hover_program) = hover_program {
                self.send_to_pointer_target(FocusPointerEvent::Hover(hover_program, hover_control)).await;
            }
        }
    }

    ///
    /// Sets the target of the pointer target, according to the pointer state
    ///
    async fn set_pointer_target(&mut self, pointer_state: &PointerState, context: &SceneContext) {
        // Do nothing if the pointer target is locked (usually by a mouse down operation)
        if self.button_state.num_buttons_down() > 0 {
            return;
        }

        // Locate the subprogram that the pointer is over
        let (target_program, target_program_control) = self.pointer_target(pointer_state.location_in_canvas);

        let pointer_target          = &mut self.pointer_target;
        let pointer_target_program  = &mut self.pointer_target_program;
        let pointer_target_control  = &mut self.pointer_target_control;

        // Connect to the program
        if let Some(target_program) = target_program {
            // Over a specific region
            if *pointer_target_program != Some(target_program) {
                // Indicate that we've left the old control (TODO: track individual pointers separately)
                let old_target_control = pointer_target_control.clone();
                if let Some(old_target) = pointer_target {
                    // Leave the control
                    old_target.send(FocusPointerEvent::Pointer(old_target_control, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();

                    if old_target_control.is_some() {
                        // Also leave the subprogram if we were in a control
                        old_target.send(FocusPointerEvent::Pointer(None, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();
                    }
                }

                // Update the pointer target
                *pointer_target = context.send(target_program).ok();

                if let Some(new_target) = pointer_target {
                    // Indicate that we've entered the new program
                    new_target.send(FocusPointerEvent::Pointer(None, PointerAction::Enter, PointerId(0), PointerState::new())).await.ok();
                }

                // No pointer target control at this point
                *pointer_target_control = None;
            }

            *pointer_target_program = Some(target_program);
        } else if let Some(canvas_program) = self.canvas_program {
            // Over the canvas
            if *pointer_target_program != Some(canvas_program) {
                // Leave the old subprogram + control
                let old_target_control = pointer_target_control.clone();
                if let Some(old_target) = pointer_target {
                    // Leave the control
                    old_target.send(FocusPointerEvent::Pointer(old_target_control, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();

                    if old_target_control.is_some() {
                        // Also leave the subprogram if we were in a control
                        old_target.send(FocusPointerEvent::Pointer(None, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();
                    }

                    *pointer_target_control = None;
                }

                *pointer_target = context.send(canvas_program).ok();

                // Enter the canvas
                if let Some(new_target) = pointer_target {
                    // Indicate that we've entered the new program
                    new_target.send(FocusPointerEvent::Pointer(None, PointerAction::Enter, PointerId(0), PointerState::new())).await.ok();
                }
            }

            *pointer_target_program = Some(canvas_program);
        } else {
            // Leave the current control
            let old_target_control = pointer_target_control.clone();
            if let Some(old_target) = pointer_target {
                // Leave the control
                old_target.send(FocusPointerEvent::Pointer(old_target_control, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();

                if old_target_control.is_some() {
                    // Also leave the subprogram if we were in a control
                    old_target.send(FocusPointerEvent::Pointer(None, PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();
                }

                *pointer_target_control = None;
            }

            // No canvas program set
            *pointer_target         = None;
            *pointer_target_program = None;
            *pointer_target_control = None;
        }

        if &target_program_control != &*pointer_target_control {
            if let (Some(old_control), Some(target)) = (&*pointer_target_control, pointer_target.as_mut()) {
                // Leave the old control
                target.send(FocusPointerEvent::Pointer(Some(old_control.clone()), PointerAction::Leave, PointerId(0), PointerState::new())).await.ok();
            }

            *pointer_target_control = target_program_control.clone();

            if let (Some(new_control), Some(target)) = (&*pointer_target_control, pointer_target.as_mut()) {
                // Enter the new control
                target.send(FocusPointerEvent::Pointer(Some(new_control.clone()), PointerAction::Enter, PointerId(0), PointerState::new())).await.ok();
            }
        }
    }

    ///
    /// Sends an event to whichever program/control is the pointer target
    ///
    async fn send_to_pointer_target(&mut self, event: FocusPointerEvent) {
        let control = self.pointer_target_control;

        if let Some(pointer_target) = &mut self.pointer_target {
            if pointer_target.send(event.with_target(control)).await.is_err() {
                self.pointer_target = None;
            }
        }
    }

    ///
    /// Sends an event to all registered programs
    ///
    async fn send_to_all(&mut self, event: impl Clone + SceneMessage, context: &SceneContext) {
        // Make a list of all the subprograms we know about
        let all_subprograms = self.subprogram_data.keys().copied()
            .chain(self.subprogram_order.iter().copied())
            .collect::<HashSet<_>>();

        // Send copies of the events to each one
        let mut send_actions = vec![];

        for program in all_subprograms {
            // Send to each program in turn
            if let Ok(mut target) = context.send(program) {
                let event = event.clone();
                send_actions.push(async move { 
                    if target.is_attached() {
                        target.send(event).await.ok(); 
                    }
                });
            }
        }

        future::join_all(send_actions).await;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::result::{Result};

    fn expect_focus(evt: FocusEvent, control: ControlId, control_num: usize) -> Result<(), String> {
        if let FocusEvent::Keyboard(FocusKeyboardEvent::Focused(actual_control)) = evt {
            if actual_control == control {
                Ok(())
            } else {
                Err(format!("Expected focus control {}, got {:?}", control_num, evt))
            }
        } else {
            Err(format!("Expected focus control {}, got {:?}", control_num, evt))
        }
    }

    fn expect_unfocus(evt: FocusEvent, control: ControlId, control_num: usize) -> Result<(), String> {
        if let FocusEvent::Keyboard(FocusKeyboardEvent::Unfocused(actual_control)) = evt {
            if actual_control == control {
                Ok(())
            } else {
                Err(format!("Expected unfocus control {}, got {:?}", control_num, evt))
            }
        } else {
            Err(format!("Expected unfocus control {}, got {:?}", control_num, evt))
        }
    }

    #[test]
    fn focus_following_control() {
        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        let control_1       = ControlId::new();
        let control_2       = ControlId::new();
        let control_3       = ControlId::new();
        let control_4       = ControlId::new();

        println!("1 = {:?}, 2 = {:?}, 3 = {:?}, 4 = {:?}", control_1, control_2, control_3, control_4);

        TestBuilder::new()
            .send_message(Focus::SetFollowingControl(test_program, control_1, control_2))
            .send_message(Focus::SetFollowingControl(test_program, control_2, control_3))
            .send_message(Focus::SetFollowingControl(test_program, control_3, control_4))

            .send_message(Focus::SetKeyboardFocus(test_program, control_1))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_1, 1))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_1, 1))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_2, 2))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_2, 2))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_3, 3))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_3, 3))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_4, 4))
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    #[test]
    fn focus_previous_control() {
        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        let control_1       = ControlId::new();
        let control_2       = ControlId::new();
        let control_3       = ControlId::new();
        let control_4       = ControlId::new();

        println!("1 = {:?}, 2 = {:?}, 3 = {:?}, 4 = {:?}", control_1, control_2, control_3, control_4);

        TestBuilder::new()
            .send_message(Focus::SetFollowingControl(test_program, control_1, control_2))
            .send_message(Focus::SetFollowingControl(test_program, control_2, control_3))
            .send_message(Focus::SetFollowingControl(test_program, control_3, control_4))

            .send_message(Focus::SetKeyboardFocus(test_program, control_4))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_4, 4))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_4, 4))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_3, 3))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_3, 3))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_2, 2))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_2, 2))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_1, 1))
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    #[test]
    fn focus_previous_control_makes_loop() {
        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        let control_1       = ControlId::new();
        let control_2       = ControlId::new();
        let control_3       = ControlId::new();
        let control_4       = ControlId::new();

        println!("1 = {:?}, 2 = {:?}, 3 = {:?}, 4 = {:?}", control_1, control_2, control_3, control_4);

        TestBuilder::new()
            .send_message(Focus::SetFollowingControl(test_program, control_1, control_2))
            .send_message(Focus::SetFollowingControl(test_program, control_2, control_3))
            .send_message(Focus::SetFollowingControl(test_program, control_3, control_4))

            .send_message(Focus::SetKeyboardFocus(test_program, control_4))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_4, 4))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_4, 4))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_3, 3))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_3, 3))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_2, 2))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_2, 2))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_1, 1))
            .send_message(Focus::FocusPrevious)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_1, 1))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_4, 4))
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    #[test]
    fn focus_following_control_makes_loop() {
        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        let control_1       = ControlId::new();
        let control_2       = ControlId::new();
        let control_3       = ControlId::new();
        let control_4       = ControlId::new();

        println!("1 = {:?}, 2 = {:?}, 3 = {:?}, 4 = {:?}", control_1, control_2, control_3, control_4);

        TestBuilder::new()
            .send_message(Focus::SetFollowingControl(test_program, control_1, control_2))
            .send_message(Focus::SetFollowingControl(test_program, control_2, control_3))
            .send_message(Focus::SetFollowingControl(test_program, control_3, control_4))

            .send_message(Focus::SetKeyboardFocus(test_program, control_1))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_1, 1))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_1, 1))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_2, 2))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_2, 2))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_3, 3))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_3, 3))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_4, 4))
            .send_message(Focus::FocusNext)
            .expect_message(move |evt: FocusEvent| expect_unfocus(evt, control_4, 4))
            .expect_message(move |evt: FocusEvent| expect_focus(evt, control_1, 1))
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    ///
    /// Checks that evt is an enter event for a subprogram and not a control
    ///
    fn expect_enter_program(evt: FocusEvent) -> Result<(), String> {
        if let FocusEvent::Pointer(FocusPointerEvent::Pointer(control_id, PointerAction::Enter, _, _)) = evt {
            if control_id.is_none() {
                Ok(())
            } else {
                Err(format!("Expected PointerAction::Enter with a 'None' control ID, got {:?}", evt))
            }
        } else {
            Err(format!("Expected PointerAction::Enter, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is an enter event for a control
    ///
    fn expect_enter_control(evt: FocusEvent, control_id: ControlId) -> Result<(), String> {
        if let FocusEvent::Pointer(FocusPointerEvent::Pointer(actual_control_id, PointerAction::Enter, _, _)) = evt {
            if actual_control_id == Some(control_id) {
                Ok(())
            } else {
                Err(format!("Expected PointerAction::Enter with a '{:?}' control ID, got {:?}", control_id, evt))
            }
        } else {
            Err(format!("Expected PointerAction::Enter, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is an enter event
    ///
    fn expect_enter(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Enter, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::Enter, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is an leave event
    ///
    fn expect_leave(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Leave, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::Leave, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a button down event
    ///
    fn expect_buttondown(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::ButtonDown, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::ButtonDown, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a button up event
    ///
    fn expect_buttonup(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::ButtonUp, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::ButtonUp, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a move event
    ///
    fn expect_move(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Move, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::Move, got {:?}", evt))
        }
    }

    #[test]
    fn mouse_click_in_region() {
        use flo_curves::arc::*;

        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        // Couple of subprograms
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct SubProgram1(FocusEvent);
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct SubProgram2(FocusEvent);
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct CanvasProgram(FocusEvent);

        impl SceneMessage for SubProgram1 { }
        impl SceneMessage for SubProgram2 { }
        impl SceneMessage for CanvasProgram { }

        // Paths for our two subprograms
        let program1_path = Circle::new(UiPoint(300.0, 500.0), 100.0).to_path();
        let program2_path = Circle::new(UiPoint(700.0, 500.0), 100.0).to_path();

        let program1 = SubProgramId::called("Subprogram1");
        let program2 = SubProgramId::called("Subprogram2");
        let canvas   = SubProgramId::called("CanvasProgram");

        // Add test programs that relay the messages
        scene.add_subprogram(program1, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Program1: {:?}", msg); context.send_message(SubProgram1(msg)).await.unwrap(); } }, 10);
        scene.add_subprogram(program2, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Program2: {:?}", msg); context.send_message(SubProgram2(msg)).await.unwrap(); } }, 10);
        scene.add_subprogram(canvas, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Canvas: {:?}", msg); context.send_message(CanvasProgram(msg)).await.unwrap(); } }, 10);

        // Create some points for mouse events
        let mut in_program1_path = PointerState::new();
        let mut in_program2_path = PointerState::new();
        let mut on_canvas        = PointerState::new();

        in_program1_path.location_in_canvas = Some((300.0, 500.0));
        in_program2_path.location_in_canvas = Some((700.0, 500.0));
        on_canvas.location_in_canvas        = Some((390.0, 590.0));

        let mut in_program1_path_button_down = in_program1_path.clone();
        let mut in_program2_path_button_down = in_program1_path.clone();
        let mut on_canvas_button_down        = in_program1_path.clone();

        in_program1_path_button_down.buttons = vec![Button::Left];
        in_program2_path_button_down.buttons = vec![Button::Left];
        on_canvas_button_down.buttons        = vec![Button::Left];

        TestBuilder::new()
            .send_message(Focus::ClaimRegion { program: program1, region: vec![program1_path], z_index: 0 })
            .send_message(Focus::ClaimRegion { program: program2, region: vec![program2_path], z_index: 1 })
            .send_message(Focus::SetCanvas(canvas))

            // Should keep tracking the mouse after the button goes down as staying in program 1
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonDown, PointerId(0), in_program1_path_button_down.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttondown(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path_button_down.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonUp, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttonup(evt))

            // Moves should get sent to whichever program they're over
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt))
            .expect_message(move |SubProgram2(evt): SubProgram2| expect_enter(evt))
            .expect_message(move |SubProgram2(evt): SubProgram2| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram2(evt): SubProgram2| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram2(evt): SubProgram2| expect_leave(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))

            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), on_canvas.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt))
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_enter(evt))
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), on_canvas.clone())))
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_move(evt))

            .run_in_scene(&scene, test_program);        // No threads because otherwise the switch between programs is unreliable
    }

    #[test]
    fn mouse_click_in_control_region() {
        use flo_curves::arc::*;

        let test_program    = SubProgramId::called("focus_following_control");
        let scene           = Scene::default();

        // Couple of subprograms
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct SubProgram1(FocusEvent);
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct SubProgram2(FocusEvent);
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct CanvasProgram(FocusEvent);

        impl SceneMessage for SubProgram1 { }
        impl SceneMessage for SubProgram2 { }
        impl SceneMessage for CanvasProgram { }

        // Paths for our two subprograms
        let program1_path = Circle::new(UiPoint(300.0, 500.0), 100.0).to_path();
        let program2_path = Circle::new(UiPoint(700.0, 500.0), 100.0).to_path();

        let program1 = SubProgramId::called("Subprogram1");
        let program2 = SubProgramId::called("Subprogram2");
        let canvas   = SubProgramId::called("CanvasProgram");

        let control1 = ControlId::new();
        let control2 = ControlId::new();

        // Add test programs that relay the messages
        scene.add_subprogram(program1, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Program1: {:?}", msg); context.send_message(SubProgram1(msg)).await.unwrap(); } }, 10);
        scene.add_subprogram(program2, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Program2: {:?}", msg); context.send_message(SubProgram2(msg)).await.unwrap(); } }, 10);
        scene.add_subprogram(canvas, |mut input, context| async move { while let Some(msg) = input.next().await { println!("Canvas: {:?}", msg); context.send_message(CanvasProgram(msg)).await.unwrap(); } }, 10);

        // Create some points for mouse events
        let mut in_program1_path = PointerState::new();
        let mut in_program2_path = PointerState::new();
        let mut on_canvas        = PointerState::new();

        in_program1_path.location_in_canvas = Some((300.0, 500.0));
        in_program2_path.location_in_canvas = Some((700.0, 500.0));
        on_canvas.location_in_canvas        = Some((390.0, 590.0));

        let mut in_program1_path_button_down = in_program1_path.clone();
        let mut in_program2_path_button_down = in_program1_path.clone();
        let mut on_canvas_button_down        = in_program1_path.clone();

        in_program1_path_button_down.buttons = vec![Button::Left];
        in_program2_path_button_down.buttons = vec![Button::Left];
        on_canvas_button_down.buttons        = vec![Button::Left];

        TestBuilder::new()
            .send_message(Focus::ClaimControlRegion { program: program1, control: control1, region: vec![program1_path], z_index: 0 })
            .send_message(Focus::ClaimControlRegion { program: program1, control: control2, region: vec![program2_path], z_index: 1 })
            .send_message(Focus::SetCanvas(canvas))

            // Should keep tracking the mouse after the button goes down as staying in program 1
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonDown, PointerId(0), in_program1_path_button_down.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_program(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_control(evt, control1))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttondown(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path_button_down.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonUp, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttonup(evt))

            // Moves should get sent to whichever program they're over
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_control(evt, control2))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_control(evt, control1))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_move(evt))

            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), on_canvas.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt)) // Leave control
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_leave(evt)) // Leave program
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_enter(evt)) // Enter canvas
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_move(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), on_canvas.clone())))
            .expect_message(move |CanvasProgram(evt): CanvasProgram| expect_move(evt))

            .run_in_scene(&scene, test_program);        // No threads because otherwise the switch between programs is unreliable
    }
}
