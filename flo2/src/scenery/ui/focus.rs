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
/// Helps manage the state for the focus subprogram
///
struct FocusProgram {
    /// The program that canvas events get sent to (events that aren't for any region)
    canvas_program: Option<SubProgramId>,

    /// x-oriented 1D scan space for subprogram regions
    subprogram_space: Space1D<SubProgramId>,

    /// The data for each subprogram region
    subprogram_data: HashMap<SubProgramId, SubProgramRegion>,

    /// The subprogram that currently has keyboard focus
    focused_subprogram: Option<SubProgramId>,

    /// The control within the subprogram that has keyboard focus
    focused_control: Option<ControlId>,

    /// The tab ordering for the controls within this program
    tab_ordering: HashMap<SubProgramId, KeyboardSubProgram>,
}

impl FocusProgram {
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
        }
    }

    ///
    /// Sets the following control for keyboard focus (inserting control_id before next_control_id)
    ///
    async fn set_following_control(&mut self, program_id: SubProgramId, control_id: ControlId, next_control_id: ControlId) {
        // Find the subprogram for these controls
        let controls_for_program = self.tab_ordering.entry(program_id)
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

        let first_control   = controls_for_program.first_control.unwrap();
        let last_control    = controls_for_program.controls.get(&first_control).map(|first| first.previous).unwrap_or(first_control);

        // Get the existing previous control ID, if it exists
        let previous_control_id = {
            let next_control = controls_for_program.controls.entry(next_control_id)
                .or_insert_with(|| KeyboardControl {
                    control_id: next_control_id,
                    next:       first_control,
                    previous:   last_control,
                });

            let previous_control    = next_control.previous;
            next_control.previous   = control_id;

            if previous_control == last_control {
                if let Some(first_control) = controls_for_program.controls.get_mut(&first_control) {
                    first_control.previous = next_control_id;
                }
            }

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
    }

    ///
    /// Sets the following subprogram for keyboard focus (inserting program_id before next_program_id)
    ///
    async fn set_following_subprogram(&mut self, program_id: SubProgramId, next_program_id: SubProgramId) {
        // Get the existing previous program ID
        let previous_program_id = {
            let next_program = self.tab_ordering.entry(next_program_id)
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
        let current_program = self.tab_ordering.entry(program_id)
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
        if let Some(previous_program) = self.tab_ordering.get_mut(&previous_program_id) {
            previous_program.next = program_id;
        }
    }

    ///
    /// Figures out the following control
    ///
    fn next_control(&self, current_program: Option<SubProgramId>, current_control: Option<ControlId>) -> Option<(SubProgramId, ControlId)> {
        let current_program = current_program?;
        let current_control = current_control?;

        // Get the data for the current control
        let program_data = self.tab_ordering.get(&current_program)?;
        let control_data = program_data.controls.get(&current_control)?;

        // If this would loop back to the beginning then focus the next subprogram
        if Some(control_data.next) == program_data.first_control {
            let next_program    = self.tab_ordering.get(&program_data.next)?;
            let first_control   = next_program.first_control?;

            Some((program_data.next, first_control))
        } else {
            // Just the next control in the same program
            Some((current_program, control_data.next))
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
    /// Figures out the preceding control
    ///
    fn previous_control(&self, current_program: Option<SubProgramId>, current_control: Option<ControlId>) -> Option<(SubProgramId, ControlId)> {
        let current_program = current_program?;
        let current_control = current_control?;

        // Get the data for the current control
        let program_data = self.tab_ordering.get(&current_program)?;
        let control_data = program_data.controls.get(&current_control)?;
        let last_control = program_data.controls.get(&program_data.first_control?)?.previous;

        // If this would loop back to the end then focus the previous subprogram
        if control_data.previous == last_control {
            let previous_program    = self.tab_ordering.get(&program_data.previous)?;
            let last_control        = previous_program.controls.get(&previous_program.first_control?)?.previous;

            Some((program_data.previous, last_control))
        } else {
            // Just the previous control in the same program
            Some((current_program, control_data.previous))
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
            // TODO: focus the last control in the first program
        }
    }

    ///
    /// Sets the program where mouse events go if there's no region defined
    ///
    async fn set_canvas(&mut self, canvas_program: SubProgramId) {
        self.canvas_program = Some(canvas_program);
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
        canvas_program:     None,
        subprogram_space:   Space1D::empty(),
        subprogram_data:    HashMap::new(),
        focused_subprogram: None,
        focused_control:    None,
        tab_ordering:       HashMap::new(),
    };

    while let Some(request) = input.next().await {
        use Focus::*;

        match request {
            Event(event) => { todo!() },

            Update(scene_update) => { /* todo!() */ },

            // Keyboard handling
            SetKeyboardFocus(program_id, control_id)                        => focus.set_keyboard_focus(program_id, control_id, &context).await,
            SetFollowingControl(program_id, control_id, next_control_id)    => focus.set_following_control(program_id, control_id, next_control_id).await,
            SetFollowingSubProgram(program_id, next_program_id)             => focus.set_following_subprogram(program_id, next_program_id).await,
            FocusNext                                                       => focus.focus_next(&context).await,
            FocusPrevious                                                   => focus.focus_previous(&context).await,

            // Control handling
            RemoveClaim(program_id)                                     => { todo!() },
            RemoveControlClaim(program_id, control_id)                  => { todo!() },
            SetCanvas(canvas_program_id)                                => focus.set_canvas(canvas_program_id).await,
            ClaimRegion { program, region, z_index }                    => { todo!() },
            ClaimControlRegion { program, region, control, z_index }    => { todo!() },
        }
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
}
