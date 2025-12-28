use super::control_id::*;
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
/// Messages that the focus subprogram can send to focused subprograms
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusEvent {
    /// An event has occurred for the specified control
    Event(Option<ControlId>, DrawEvent),

    /// The specified control ID has received keyboard focus
    Focused(ControlId),

    /// The specified control ID has lost keyboard focus (when focus moves, we unfocus first)
    Unfocused(ControlId),
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
        pointer_target_lock_count:  0,
        focused_subprogram:         None,
        focused_control:            None,
        focused_event_target:       None,
        tab_ordering:               HashMap::new(),
        scale:                      None,
        bounds:                     None,
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
            Event(DrawEvent::Closed)                => { focus.send_to_all(DrawEvent::Closed, &context).await; }

            // Pointer and key events
            Event(DrawEvent::Pointer(PointerAction::Enter, _, _))                           => { },
            Event(DrawEvent::Pointer(PointerAction::Leave, _, _))                           => { },
            Event(DrawEvent::Pointer(PointerAction::ButtonDown, pointer_id, pointer_state)) => { focus.set_pointer_target(&pointer_state, &context).await; focus.pointer_target_lock_count += 1; focus.send_to_pointer_target(DrawEvent::Pointer(PointerAction::ButtonDown, pointer_id, pointer_state)).await; },
            Event(DrawEvent::Pointer(PointerAction::ButtonUp, pointer_id, pointer_state))   => { focus.pointer_target_lock_count -= 1; focus.send_to_pointer_target(DrawEvent::Pointer(PointerAction::ButtonUp, pointer_id, pointer_state)).await; },
            Event(DrawEvent::Pointer(other_action, pointer_id, pointer_state))              => { focus.set_pointer_target(&pointer_state, &context).await; focus.send_to_pointer_target(DrawEvent::Pointer(other_action, pointer_id, pointer_state)).await; },
            Event(DrawEvent::KeyDown(scancode, key))                                        => { focus.send_to_focus(DrawEvent::KeyDown(scancode, key)).await; },
            Event(DrawEvent::KeyUp(scancode, key))                                          => { focus.send_to_focus(DrawEvent::KeyUp(scancode, key)).await; },

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

impl SceneMessage for FocusEvent {
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
    pointer_target: Option<OutputSink<FocusEvent>>,

    /// The subprogram ID of the active pointer target
    pointer_target_program: Option<SubProgramId>,

    /// The control of the active pointer target
    pointer_target_control: Option<ControlId>,

    /// >0 if the pointer target is locked (eg, due to a mouse down that hasn't yet been matched by a mouse up)
    pointer_target_lock_count: usize,

    /// The subprogram that currently has keyboard focus
    focused_subprogram: Option<SubProgramId>,

    /// The control within the subprogram that has keyboard focus
    focused_control: Option<ControlId>,

    /// Where keyboard events should be sent
    focused_event_target: Option<OutputSink<FocusEvent>>,

    /// The tab ordering for the controls within this program
    tab_ordering: HashMap<SubProgramId, KeyboardSubProgram>,

    /// The focus order for the subprograms
    subprogram_order: Vec<SubProgramId>,

    /// The bounding box of the window (None if this has not been sent to us)
    bounds: Option<(f64, f64)>,

    /// The scale of the window (None if this has not been sent to us)
    scale: Option<f64>,
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
        self.send_to_all(DrawEvent::Scale(scale), &context).await;
    }

    ///
    /// Sets the bounds of the window
    ///
    async fn set_bounds(&mut self, width: f64, height: f64, context: &SceneContext) {
        self.bounds = Some((width, height));
        self.send_to_all(DrawEvent::Resize(width, height), &context).await;
    }

    ///
    /// Sends greeting messages to a newly added subprogram
    ///
    async fn greet_new_subprogram(&mut self, subprogram_id: SubProgramId, context: &SceneContext) {
        let subprogram = context.send(subprogram_id).ok();

        if let Some(mut subprogram) = subprogram {
            // Send the scale if it's been stored
            if let Some(scale) = self.scale {
                subprogram.send(FocusEvent::Event(None, DrawEvent::Scale(scale))).await.ok();
            }

            // Send the bounds if they've been stored
            if let Some((w, h)) = self.bounds {
                subprogram.send(FocusEvent::Event(None, DrawEvent::Resize(w, h))).await.ok();
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
                channel.send(FocusEvent::Unfocused(old_control_id)).await.ok();
            }

            self.focused_subprogram  = None;
            self.focused_control     = None;
        }

        // Update the focused control and inform the relevant program
        if let Ok(mut channel) = context.send(program_id) {
            self.focused_subprogram  = Some(program_id);
            self.focused_control     = Some(control_id);
            channel.send(FocusEvent::Focused(control_id)).await.ok();

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
    async fn send_to_focus(&mut self, event: DrawEvent) {
        let control = self.focused_control;

        if let Some(focus_target) = &mut self.focused_event_target {
            if focus_target.send(FocusEvent::Event(control, event)).await.is_err() {
                self.focused_event_target = None;
            }
        }
    }

    ///
    /// Sets the target of the pointer target, according to the pointer state
    ///
    async fn set_pointer_target(&mut self, pointer_state: &PointerState, context: &SceneContext) {
        // Do nothing if the pointer target is locked (usually by a mouse down operation)
        if self.pointer_target_lock_count > 0 {
            return;
        }

        let space                   = &mut self.subprogram_space;
        let subprogram_data         = &self.subprogram_data;
        let pointer_target          = &mut self.pointer_target;
        let pointer_target_program  = &mut self.pointer_target_program;
        let pointer_target_control  = &mut self.pointer_target_control;

        // Generate the space if it's not already generated
        let space = if let Some(space) = space {
            space
        } else {
            *space = Some(Space1D::from_data(subprogram_data.iter().map(|(program_id, region)| (region.bounds.min().x()..region.bounds.max().x(), *program_id))));
            space.as_mut().unwrap()
        };

        // Locate the subprogram that the pointer is over
        let target_program = if let Some((x, y)) = pointer_state.location_in_canvas {
            // Find all of the subprograms where the point might be inside
            let mut possible_matches = space.data_at_point(x)
                .flat_map(|subprogram_id| subprogram_data.get(subprogram_id).map(|region| (subprogram_id, region)))
                .filter(|(_, region)| UiPoint(x, y).in_bounds(&region.bounds))
                .filter(|(_, region)| region.point_is_inside(x, y))
                .collect::<Vec<_>>();

            // Order by z-index if there are multiple possibilities
            if possible_matches.len() > 1 {
                possible_matches.sort_by_key(|(_, region)| region.z_index);
            }

            // Highest z index is the target program
            possible_matches.last().map(|(program_id, _)| **program_id)
        } else {
            None
        };

        // Connect to the program
        if let Some(target_program) = target_program {
            // Over a specific region
            if *pointer_target_program != Some(target_program) {
                // Indicate that we've left the old control (TODO: track individual pointers separately)
                let old_target_control = pointer_target_control.clone();
                if let Some(old_target) = pointer_target {
                    // Leave the control
                    old_target.send(FocusEvent::Event(old_target_control, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();

                    if old_target_control.is_some() {
                        // Also leave the subprogram if we were in a control
                        old_target.send(FocusEvent::Event(None, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();
                    }
                }

                // Update the pointer target
                *pointer_target = context.send(target_program).ok();

                if let Some(new_target) = pointer_target {
                    // Indicate that we've entered the new program
                    new_target.send(FocusEvent::Event(None, DrawEvent::Pointer(PointerAction::Enter, PointerId(0), PointerState::new()))).await.ok();
                }
            }

            *pointer_target_program = Some(target_program);
        } else if let Some(canvas_program) = self.canvas_program {
            // Over the canvas
            if *pointer_target_program != Some(canvas_program) {
                // Leave the old subprogram + control
                let old_target_control = pointer_target_control.clone();
                if let Some(old_target) = pointer_target {
                    // Leave the control
                    old_target.send(FocusEvent::Event(old_target_control, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();

                    if old_target_control.is_some() {
                        // Also leave the subprogram if we were in a control
                        old_target.send(FocusEvent::Event(None, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();
                    }
                }

                *pointer_target = context.send(canvas_program).ok();

                // Enter the canvas
                if let Some(new_target) = pointer_target {
                    // Indicate that we've entered the new program
                    new_target.send(FocusEvent::Event(None, DrawEvent::Pointer(PointerAction::Enter, PointerId(0), PointerState::new()))).await.ok();
                }
            }

            *pointer_target_program = Some(canvas_program);
        } else {
            // Leave the current control
            let old_target_control = pointer_target_control.clone();
            if let Some(old_target) = pointer_target {
                // Leave the control
                old_target.send(FocusEvent::Event(old_target_control, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();

                if old_target_control.is_some() {
                    // Also leave the subprogram if we were in a control
                    old_target.send(FocusEvent::Event(None, DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();
                }
            }

            // No canvas program set
            *pointer_target         = None;
            *pointer_target_program = None;
        }

        if let (Some(target_program), Some((x, y))) = (target_program, pointer_state.location_in_canvas) {
            let target_program_data = subprogram_data.get(&target_program);

            if let Some(target_program_data) = target_program_data {
                // Find the control that the point might be inside
                let mut possible_controls = target_program_data.controls.iter()
                    .filter(|control| UiPoint(x, y).in_bounds(&control.bounds))
                    .filter(|control| control.point_is_inside(x, y))
                    .collect::<Vec<_>>();

                // Order by z-index if there are multiple possibilities
                if possible_controls.len() > 1 {
                    possible_controls.sort_by_key(|control| control.z_index);
                }

                // The highest z-index is the target control
                let new_control = possible_controls.last().map(|control| control.id);

                if &new_control != &*pointer_target_control {
                    if let (Some(old_control), Some(target)) = (&*pointer_target_control, pointer_target.as_mut()) {
                        // Leave the old control
                        target.send(FocusEvent::Event(Some(old_control.clone()), DrawEvent::Pointer(PointerAction::Leave, PointerId(0), PointerState::new()))).await.ok();
                    }

                    *pointer_target_control = new_control.clone();

                    if let (Some(new_control), Some(target)) = (&*pointer_target_control, pointer_target.as_mut()) {
                        // Enter the new control
                        target.send(FocusEvent::Event(Some(new_control.clone()), DrawEvent::Pointer(PointerAction::Enter, PointerId(0), PointerState::new()))).await.ok();
                    }
                }
            } else {
                *pointer_target_control = None;
            }
        } else {
            // Target is the canvas
            *pointer_target_control = None;
        }
    }

    ///
    /// Sends an event to whichever program/control is the pointer target
    ///
    async fn send_to_pointer_target(&mut self, event: DrawEvent) {
        let control = self.pointer_target_control;

        if let Some(pointer_target) = &mut self.pointer_target {
            if pointer_target.send(FocusEvent::Event(control, event)).await.is_err() {
                self.pointer_target = None;
            }
        }
    }

    ///
    /// Sends an event to all registered programs
    ///
    async fn send_to_all(&mut self, event: DrawEvent, context: &SceneContext) {
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
                        target.send(FocusEvent::Event(None, event)).await.ok(); 
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
        if let FocusEvent::Focused(actual_control) = evt {
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
        if let FocusEvent::Unfocused(actual_control) = evt {
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
        if let FocusEvent::Event(control_id, DrawEvent::Pointer(PointerAction::Enter, _, _)) = evt {
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
        if let FocusEvent::Event(actual_control_id, DrawEvent::Pointer(PointerAction::Enter, _, _)) = evt {
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
        if matches!(evt, FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Enter, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::Enter, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is an leave event
    ///
    fn expect_leave(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Leave, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::Leave, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a button down event
    ///
    fn expect_buttondown(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonDown, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::ButtonDown, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a button up event
    ///
    fn expect_buttonup(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, _, _))) {
            Ok(())
        } else {
            Err(format!("Expected PointerAction::ButtonUp, got {:?}", evt))
        }
    }

    ///
    /// Checks that evt is a move event
    ///
    fn expect_move(evt: FocusEvent) -> Result<(), String> {
        if matches!(evt, FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, _, _))) {
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

        TestBuilder::new()
            .send_message(Focus::ClaimRegion { program: program1, region: vec![program1_path], z_index: 0 })
            .send_message(Focus::ClaimRegion { program: program2, region: vec![program2_path], z_index: 1 })
            .send_message(Focus::SetCanvas(canvas))

            // Should keep tracking the mouse after the button goes down as staying in program 1
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonDown, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttondown(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
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

        TestBuilder::new()
            .send_message(Focus::ClaimControlRegion { program: program1, control: control1, region: vec![program1_path], z_index: 0 })
            .send_message(Focus::ClaimControlRegion { program: program1, control: control2, region: vec![program2_path], z_index: 1 })
            .send_message(Focus::SetCanvas(canvas))

            // Should keep tracking the mouse after the button goes down as staying in program 1
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::ButtonDown, PointerId(0), in_program1_path.clone())))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_program(evt))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_enter_control(evt, control1))
            .expect_message(move |SubProgram1(evt): SubProgram1| expect_buttondown(evt))
            .send_message(Focus::Event(DrawEvent::Pointer(PointerAction::Move, PointerId(0), in_program2_path.clone())))
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
