//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use super::tool_state::*;

use flo_scene::*;
use flo_draw::*;

use futures::prelude::*;
use serde::*;

///
/// Message sent to a tool dock
///
#[derive(Serialize, Deserialize)]
pub enum ToolDockMessage {
    /// Updating the tool state for this dock
    ToolState(ToolState),

    /// Drawing event for the window this dock is in
    DrawEvent(DrawEvent),
}

impl SceneMessage for ToolDockMessage {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|tool_state_msgs| tool_state_msgs.map(|msg| ToolDockMessage::ToolState(msg)))), (), StreamId::with_message_type::<ToolState>())
            .unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|draw_event_msgs| draw_event_msgs.map(|msg| ToolDockMessage::DrawEvent(msg)))), (), StreamId::with_message_type::<DrawEvent>())
            .unwrap();
    }
}

///
/// Where the dock should appear in the window
///
pub enum DockPosition {
    Left,
    Right,
}

///
/// Runs a tool dock subprogram. This is a location, which can be used with the `Tool::SetToolLocation` message to specify which tools are found in this dock.
/// 
/// In order to draw the tool dock at the correct size, this requires `DrawEvent` messages (this does not subscribe to these itself, but
/// this can be set up by sending `DocumentRequest::SubscribeDrawEvents` for example)
///
pub async fn tool_dock_program(input: InputStream<ToolDockMessage>, context: SceneContext, position: DockPosition) {
    // Run the program
    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            ToolDockMessage::ToolState(_) => { /* Other toolstate messages are ignored */ }
            ToolDockMessage::DrawEvent(_) => { /* Other drawing events are ignored */ }
        }
    }
}
