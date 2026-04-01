// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::scenery::ui::*;
use crate::scenery::canvas::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::*;

use futures::prelude::*;
use futures::stream::FuturesUnordered;

use std::collections::*;
use std::sync::*;

use serde::*;

///
/// Shared state for the two programs that make up the relay
///
struct ActiveTools {
    // Current set of active tool programs
    active_tool_programs: HashSet<SubProgramId>,

    // Transform applied to the tool layer
    inverse_layer_transform: Transform2D,
}

///
/// Requests to the tool canvas program to indicate which tools are selected or not
///
#[derive(Serialize, Deserialize)]
pub enum FocusTool {
    /// Indicates that a tool is selected
    SelectedTool(SubProgramId),

    /// Indicates that a tool is unselected
    DeselectedTool(SubProgramId),
}

impl SceneMessage for FocusTool {
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::tool::canvas_focus").into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Run the FocusTool program
        init_context.add_subprogram(SubProgramId::called("flowbetween::tool::canvas_focus"), focus_tool_program, 1);

        // Create the FocusTool program
        init_context.connect_programs((), SubProgramId::called("flowbetween::tool::canvas_focus"), StreamId::with_message_type::<FocusTool>()).unwrap();
    }
}

///
/// Subprogram that distributes focus events sent to the main canvas to the currently selected
/// tools.
///
pub async fn focus_tool_program(input: InputStream<FocusTool>, context: SceneContext) {
    let Some(our_program_id) = context.current_program_id() else { return; };

    let active_tools = Arc::new(Mutex::new(ActiveTools { active_tool_programs: HashSet::new(), inverse_layer_transform: Transform2D::identity() }));

    // Tell SceneControl to start a child program to do the actual event relaying (while we update which tools are active)
    let relay_program_id    = SubProgramId::new();
    let relay_state         = active_tools.clone();
    context.send_message(SceneControl::start_child_program(relay_program_id, our_program_id, move |input, context| tool_canvas_relay(input, context, relay_state), 20)).await.ok();

    // Tell SceneControl to monitor the layer transform so we can transform the coordinates sent in focus events
    let canvas_state_program_id = SubProgramId::new();
    let canvas_state_state      = active_tools.clone();
    context.send_message(SceneControl::start_child_program(canvas_state_program_id, our_program_id, move |input, context| tool_canvas_state_tracker(input, context, canvas_state_state), 20)).await.ok();

    // Tell Focus that our child program owns the canvas
    context.send_message(Focus::SetCanvas(relay_program_id)).await.ok();

    // Update state on request
    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            FocusTool::SelectedTool(program_id)     => { active_tools.lock().unwrap().active_tool_programs.insert(program_id); }
            FocusTool::DeselectedTool(program_id)   => { active_tools.lock().unwrap().active_tool_programs.remove(&program_id); }
        }
    }
}

///
/// Subprogram that forwards focus events to the active tool porgrams
///
async fn tool_canvas_relay(input: InputStream<FocusEvent>, context: SceneContext, state: Arc<Mutex<ActiveTools>>) {
    // The connections for each program (we add new connections the first time we encounter a subprogram)
    let mut connections = HashMap::new();

    // Event input
    let mut input = input;
    while let Some(event) = input.next().await {
        // Update the event with canvas transforms
        let event = match event {
            FocusEvent::Pointer(FocusPointerEvent::Pointer(control_id, pointer_action, pointer_id, pointer_state)) => {
                let mut pointer_state = pointer_state;

                if let Some(location_in_canvas) = &mut pointer_state.location_in_canvas {
                    // Apply the location in canvas transform
                    let transform = state.lock().unwrap().inverse_layer_transform;

                    let (x, y) = transform.transform_point(location_in_canvas.0 as _, location_in_canvas.1 as _);
                    *location_in_canvas = (x as _, y as _);
                }

                FocusEvent::Pointer(FocusPointerEvent::Pointer(control_id, pointer_action, pointer_id, pointer_state))
            },

            event => event
        };

        // Ensure we're connected to all active programs
        let active_programs = state.lock().unwrap().active_tool_programs.clone();

        for program in active_programs.iter() {
            // Add a new connection for this program
            if !connections.contains_key(program) {
                let connection = context.send(*program).ok();
                connections.insert(*program, connection);
            }
        }

        // Send the event to each connection (taking the connections while we do this: need to do some juggling to satisfy the borrow checker)
        let mut futures = vec![];

        for program in active_programs.iter() {
            if let Some(connection) = connections.remove(program) {
                let event   = event.clone();
                let program = *program;

                futures.push(async move { 
                    if let Some(mut connection) = connection {
                        if connection.send(event).await.is_ok() {
                            // Restore the connection after we're done
                            (program, Some(connection))
                        } else {
                            // Connection failed, don't try to talk to this tool again
                            (program, None)
                        }
                    } else {
                        // Tool has no active connection
                        (program, None)
                    }
                });
            }
        }

        // Restore the connections to the list as the futures finish, ready for the next event
        let mut returning_connections = futures.into_iter().collect::<FuturesUnordered<_>>();
        while let Some((program, maybe_connection)) = returning_connections.next().await {
            connections.insert(program, maybe_connection);
        }
    }
}

///
/// Tracks the current layer transform for the canvas layers
///
async fn tool_canvas_state_tracker(input: InputStream<CanvasRenderUpdate>, context: SceneContext, data: Arc<Mutex<ActiveTools>>) {
    let our_program_id = context.current_program_id().unwrap();

    // Request updates on the layer transform
    context.send_message(CanvasRender::Subscribe(our_program_id.into())).await.ok();

    // Monitor for updates
    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            CanvasRenderUpdate::LayerTransform(transform)   => { data.lock().unwrap().inverse_layer_transform = transform.invert().unwrap_or_else(|| Transform2D::identity()); },
            CanvasRenderUpdate::Layers(_layers)             => { },
        }
    }
}
