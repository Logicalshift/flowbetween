//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use crate::scenery::ui::*;
use super::tool_state::*;

use flo_scene::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;

use futures::prelude::*;
use serde::*;

use std::collections::*;
use std::sync::*;

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
/// Data attached to a tool in the dock
///
struct ToolData {
    position:   (f64, f64),
    icon:       Arc<Vec<Draw>>,
    sprite:     Option<SpriteId>,
    selected:   bool,
}

///
/// Data storage for the tool dock
///
struct ToolDock {
    /// Where in the window to draw the dock
    position:       DockPosition,

    /// The layer that the dock is drawn on
    layer:          LayerId,

    /// The namespace that the dock is drawn in
    namespace:      NamespaceId,

    /// The tools that are stored in this dock
    tools:          HashMap<ToolId, ToolData>,

    /// Sprite IDs that were used by tools but are no longer in use
    unused_sprites: Vec<SpriteId>,

    /// Next sprite ID to use if there are no sprites in the unused pool
    next_sprite:    SpriteId,
}

impl ToolDock {
    ///
    /// Redraws this tool dock
    ///
    pub fn draw(&self, gc: &mut impl GraphicsContext, window_size: (f64, f64)) {
        // Store the GC state and reset it to the layer this dock is drawn on
        gc.push_state();
        gc.namespace(self.namespace);
        gc.layer(self.layer);
        gc.clear_layer();

        // Finish up by clearing the state
        gc.pop_state();
    }
}

///
/// Runs a tool dock subprogram. This is a location, which can be used with the `Tool::SetToolLocation` message to specify which tools are found in this dock.
/// 
/// In order to draw the tool dock at the correct size, this requires `DrawEvent` messages (this does not subscribe to these itself, but
/// this can be set up by sending `DocumentRequest::SubscribeDrawEvents` for example)
///
pub async fn tool_dock_program(input: InputStream<ToolDockMessage>, context: SceneContext, position: DockPosition, layer: LayerId) {
    // Tool dock data
    let mut tool_dock = ToolDock {
        position:       position,
        layer:          layer,
        namespace:      *DOCK_LAYER,
        tools:          HashMap::new(),
        unused_sprites: vec![],
        next_sprite:    SpriteId(0),
    };

    // Size of the viewport
    let mut scale   = 1.0;
    let mut w       = 0.0;
    let mut h       = 0.0;

    // Run the program
    let mut input = input.ready_chunks(50);
    while let Some(msgs) = input.next().await {
        let mut needs_redraw = false;

        // Process the messages that are waiting
        for msg in msgs {
            match msg {
                ToolDockMessage::DrawEvent(DrawEvent::Resize(new_w, new_h)) => {
                    // Update the width and height
                    w = new_w / scale;
                    h = new_h / scale;

                    needs_redraw = true;
                }

                ToolDockMessage::DrawEvent(DrawEvent::Scale(new_scale)) => {
                    // Update the scale, adjust the width and height accordingly
                    w = (w * scale) / new_scale;
                    h = (h * scale) / new_scale;
                    scale = new_scale;

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::AddTool(tool_id)) => { 
                    // Add (or replace) the tool with this ID
                    tool_dock.tools.insert(tool_id, ToolData {
                        position:   (0.0, 0.0),
                        icon:       Arc::new(vec![]),
                        sprite:     None,
                        selected:   false,
                    });

                    // Draw with the new tool
                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::SetIcon(tool_id, icon)) => {
                    // Update the icon
                    if let Some(tool) = tool_dock.tools.get_mut(&tool_id) {
                        tool.icon = icon;

                        if let Some(old_sprite) = tool.sprite.take() {
                            // Remove the sprite to force the tool to redraw its icon
                            tool_dock.unused_sprites.push(old_sprite);
                        }
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::LocateTool(tool_id, position)) => {
                    // Change the position (we use the y position to set the ordering in the dock)0
                    if let Some(tool) = tool_dock.tools.get_mut(&tool_id) {
                        tool.position = position;
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::RemoveTool(tool_id)) => {
                    // Remove the tool from this dock
                    if let Some(mut old_tool) = tool_dock.tools.remove(&tool_id) {
                        if let Some(old_sprite) = old_tool.sprite.take() {
                            tool_dock.unused_sprites.push(old_sprite);
                        }
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::Select(tool_id)) => {
                    // Mark this tool as selected
                    if let Some(tool) = tool_dock.tools.get_mut(&tool_id) {
                        tool.selected = true;
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::Deselect(tool_id)) => {
                    // Mark this tool as unselected
                    if let Some(tool) = tool_dock.tools.get_mut(&tool_id) {
                        tool.selected = false;
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(_) => { /* Other toolstate messages are ignored */ }
                ToolDockMessage::DrawEvent(_) => { /* Other drawing events are ignored */ }
            }
        }

        // Redraw the dock if necessary
        if needs_redraw {
            let mut drawing = vec![];
            tool_dock.draw(&mut drawing, (w, h));

            context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}
