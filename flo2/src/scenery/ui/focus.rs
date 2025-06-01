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
/// Represents the ordering of subprograms that can receive keyboard focus
///
struct KeyboardSubProgram {
    subprogram_id:  SubProgramId,
    control_order:  Vec<ControlId>,
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

    /// The focus order for the subprograms
    subprogram_order: Vec<SubProgramId>,

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
        if !self.subprogram_order.iter().any(|prog| prog == &program_id) {
            self.subprogram_order.push(program_id);
        }

        let controls_for_program = self.tab_ordering.entry(program_id)
            .or_insert_with(|| KeyboardSubProgram {
                subprogram_id:  program_id,
                control_order:  vec![],
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
                subprogram_id:  program_id,
                control_order:  vec![],
            });
        self.tab_ordering.entry(next_program_id)
            .or_insert_with(|| KeyboardSubProgram {
                subprogram_id:  program_id,
                control_order:  vec![],
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
        subprogram_order:   vec![],
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
}
