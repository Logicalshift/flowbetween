use super::control_id::*;
use super::subprograms::*;
use super::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::*;
use flo_curves::geo::*;
use flo_curves::bezier::path::*;

use futures::prelude::*;
use serde::*;

use std::collections::{HashMap};

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
    SetFollowingControl(SubProgramId, ControlId, ControlId),

    /// Sets which subprogram should receive keyboard focus after reaching the end of the controls in the specified 
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

    /// Claims a region for a single control within the region for a subprogram. The z-index here is used 
    ClaimControlRegion { program: SubProgramId, region: Vec<UiPath>, control: ControlId, z_index: usize },

    /// Removes a claim added by ClaimRegion
    RemoveClaim(SubProgramId),

    /// Removes a claim added by ClaimControlRegion
    RemoveControlClaim(SubProgramId, ControlId),
}

///
/// Messages that the focus subprogram can send to focused subprograms
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FocusEvent {
    /// An event has occurred for the specified control
    Event(Option<ControlId>, DrawEvent),

    /// The specified control ID has received keyboard focus
    Focused(ControlId),

    /// The specified control ID has lost keyboard focus (when focus moves, we unfocus first)
    Unfocused(ControlId),
}

impl SceneMessage for Focus {
    fn default_target() -> StreamTarget {
        subprogram_focus().into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Set up filters for the focus events/updates
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|scene_updates| scene_updates.map(|update| Focus::Update(update)))), (), StreamId::with_message_type::<SceneUpdate>());
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|draw_events| draw_events.map(|event| Focus::Event(event)))), (), StreamId::with_message_type::<DrawEvent>());

        // Create the standard focus subprogram when a message is sent for the first tiem
        init_context.add_subprogram(subprogram_focus(), focus, 20);

        // This is the default target for focus messages to this scene
        init_context.connect_programs((), subprogram_focus(), StreamId::with_message_type::<Focus>());
    }
}

impl SceneMessage for FocusEvent {
}

///
/// Represents a control within a subprogram
///
struct SubProgramControl {
    id:         ControlId,
    region:     Vec<UiPath>,
    z_index:    usize,
}

///
/// Definition for a region of the canvas where a subprogram owns the events
///
struct SubProgramRegion {
    region:     Vec<UiPath>,
    bounds:     Bounds<UiPoint>,
    controls:   Vec<SubProgramControl>,
    z_index:    usize,
}

///
/// Represents the ordering of controls that can receive keyboard focus
///
struct KeyboardControl {
    control_id: ControlId,
    next:       ControlId,
    previous:   ControlId,
}

///
/// Represents the ordering of subprograms that can receive keyboard focus
///
struct KeyboardSubProgram {
    subprogram_id:  SubProgramId,
    first_control:  Option<ControlId>,
    controls:       HashMap<ControlId, KeyboardControl>,
    next:           SubProgramId,
    previous:       SubProgramId,
}


///
/// Runs the UI focus subprogram
///
pub async fn focus(input: InputStream<Focus>, context: SceneContext) {
    let program_id  = context.current_program_id().unwrap();
    let mut input   = input;

    // Request updates from the scene (which we'll use to remove subprograms that aren't running any more)
    context.send_message(SceneControl::Subscribe(program_id.into())).await.ok();

    // We use a 1D space to store the regions (so we do an x-sweep to find them)
    let mut canvas_program      = None;
    let mut subprogram_space    = Space1D::<SubProgramId>::empty();
    //let mut subprogram_data     = HashMap::new();

    // We also store the tab ordering of subprograms (and the controls within those subprograms)
    let mut focused_subprogram  = None;
    let mut focused_control     = None;
    let mut tab_ordering        = HashMap::new();

    while let Some(request) = input.next().await {
        use Focus::*;

        match request {
            Event(event) => { todo!() },

            Update(scene_update) => { todo!() },

            // Keyboard handling
            SetKeyboardFocus(program_id, control_id) => {
                // Unfocus the existing program
                if let (Some(old_program_id), Some(old_control_id)) = (focused_subprogram, focused_control) {
                    if let Ok(mut channel) = context.send(old_program_id) {
                        channel.send(FocusEvent::Unfocused(old_control_id)).await.ok();
                    }

                    focused_subprogram  = None;
                    focused_control     = None;
                }

                // Update the focused control and inform the relevant program
                if let Ok(mut channel) = context.send(program_id) {
                    focused_subprogram  = Some(program_id);
                    focused_control     = Some(control_id);
                    channel.send(FocusEvent::Focused(control_id)).await.ok();
                }
            },

            SetFollowingControl(program_id, control_id, next_control_id) => {
                // Insert control_id into the list before next_control_id

                // Find the subprogram for this control
                let controls_for_program = tab_ordering.entry(program_id)
                    .or_insert_with(|| KeyboardSubProgram {
                        subprogram_id:  program_id,
                        first_control:  Some(control_id),
                        controls:       HashMap::new(),
                        next:           program_id,
                        previous:       program_id,
                    });

                if controls_for_program.first_control.is_none() {
                    controls_for_program.first_control = Some(control_id);
                }

                // Get the existing previous control ID
                let previous_control_id = {
                    let next_control = controls_for_program.controls.entry(next_control_id)
                        .or_insert_with(|| KeyboardControl {
                            control_id: next_control_id,
                            next:       next_control_id,
                            previous:   next_control_id,
                        });

                    let previous_control    = next_control.previous;
                    next_control.previous   = control_id;

                    previous_control
                };

                // Get the current control
                let current_control = controls_for_program.controls.entry(control_id)
                    .or_insert_with(|| KeyboardControl {
                        control_id: control_id,
                        next:       control_id,
                        previous:   control_id,
                    });

                current_control.next        = next_control_id;
                current_control.previous    = previous_control_id;

                // Update the previous control
                if let Some(previous_control) = controls_for_program.controls.get_mut(&previous_control_id) {
                    previous_control.next = control_id;
                }
            },

            SetFollowingSubProgram(program_id, next_program_id) => {
                // Insert program_id into the list before next_program_id

                // Get the existing previous program ID
                let previous_program_id = {
                    let next_program = tab_ordering.entry(next_program_id)
                        .or_insert_with(|| KeyboardSubProgram {
                            subprogram_id:  next_program_id,
                            first_control:  None,
                            controls:       HashMap::new(),
                            next:           next_program_id,
                            previous:       next_program_id,
                        });

                    let previous_control    = next_program.previous;
                    next_program.previous   = program_id;

                    previous_control
                };

                // Get the current program
                let current_program = tab_ordering.entry(program_id)
                    .or_insert_with(|| KeyboardSubProgram {
                        subprogram_id:  program_id,
                        first_control:  None,
                        controls:       HashMap::new(),
                        next:           program_id,
                        previous:       program_id,
                    });

                current_program.next        = next_program_id;
                current_program.previous    = previous_program_id;

                // Update the previous control
                if let Some(previous_program) = tab_ordering.get_mut(&previous_program_id) {
                    previous_program.next = program_id;
                }
            },

            FocusNext => { todo!() },
            FocusPrevious => { todo!() },

            // Control handling
            RemoveClaim(program_id) => { todo!() },
            RemoveControlClaim(program_id, control_id) => { todo!() },

            SetCanvas(canvas_program_id) => {
                canvas_program = Some(canvas_program_id);
            },

            ClaimRegion { program, region, z_index } => { todo!() },
            ClaimControlRegion { program, region, control, z_index } => { todo!() },
        }
    }
}
