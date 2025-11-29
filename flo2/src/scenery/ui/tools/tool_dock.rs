//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use crate::scenery::ui::*;
use super::tool_state::*;

use flo_curves::bezier::path::*;
use flo_scene::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;

use futures::prelude::*;
use serde::*;

use std::collections::*;
use std::sync::*;

const DOCK_WIDTH: f64       = 48.0;
const DOCK_TOP_MARGIN: f64  = 100.0;
const DOCK_SIDE_MARGIN: f64 = 16.0;
const DOCK_Z_INDEX: usize   = 1000;

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

    ///
    /// Calculates the corners of the region for this tool dock in a window of the specified size
    ///
    pub fn region(&self, w: f64, h: f64) -> (UiPoint, UiPoint) {
        match self.position {
            DockPosition::Left => { 
                let topleft     = UiPoint(DOCK_SIDE_MARGIN, DOCK_TOP_MARGIN);
                let bottomright = UiPoint(DOCK_SIDE_MARGIN + DOCK_WIDTH, h - DOCK_TOP_MARGIN);

                (topleft, bottomright)
            }

            DockPosition::Right => {
                let topleft     = UiPoint(w - DOCK_SIDE_MARGIN - DOCK_WIDTH, DOCK_TOP_MARGIN);
                let bottomright = UiPoint(w - DOCK_SIDE_MARGIN, h - DOCK_TOP_MARGIN);

                (topleft, bottomright)
            }
        }
    }

    ///
    /// Creates the dock region as a UiPath
    ///
    pub fn region_as_path(&self, w: f64, h: f64) -> UiPath {
        let (topleft, bottomright) = self.region(w, h);

        BezierPathBuilder::start(topleft)
            .line_to(UiPoint(topleft.x(), bottomright.y()))
            .line_to(bottomright)
            .line_to(UiPoint(bottomright.x(), topleft.y()))
            .line_to(topleft)
            .build()
    }
}

///
/// Runs a tool dock subprogram. This is a location, which can be used with the `Tool::SetToolLocation` message to specify which tools are found in this dock.
///
pub async fn tool_dock_program(input: InputStream<ToolDockMessage>, context: SceneContext, position: DockPosition, layer: LayerId) {
    let our_program_id = context.current_program_id().unwrap();

    // The focus subprogram is used to send events to the dock
    let mut focus = context.send(()).unwrap();

    // Tool dock data
    let mut tool_dock = ToolDock {
        position:       position,
        layer:          layer,
        namespace:      *DOCK_LAYER,
        tools:          HashMap::new(),
        unused_sprites: vec![],
        next_sprite:    SpriteId(0),
    };

    // Size of the viewport (we don't know the actual size yet)
    let mut scale   = 1.0;
    let mut w       = 1000.0;
    let mut h       = 1000.0;

    // Claim an initial region (this is so the focus subprogram sends us a greeting)
    focus.send(Focus::ClaimRegion { program: our_program_id, region: vec![tool_dock.region_as_path(w, h)], z_index: DOCK_Z_INDEX }).await.ok();

    // Run the program
    let mut input = input.ready_chunks(50);
    while let Some(msgs) = input.next().await {
        let mut needs_redraw = false;
        let mut size_changed = false;

        // Process the messages that are waiting
        for msg in msgs {
            match msg {
                ToolDockMessage::DrawEvent(DrawEvent::Resize(new_w, new_h)) => {
                    // Update the width and height
                    w = new_w / scale;
                    h = new_h / scale;

                    needs_redraw = true;
                    size_changed = true;
                }

                ToolDockMessage::DrawEvent(DrawEvent::Scale(new_scale)) => {
                    // Update the scale, adjust the width and height accordingly
                    w = (w * scale) / new_scale;
                    h = (h * scale) / new_scale;
                    scale = new_scale;

                    needs_redraw = true;
                    size_changed = true;
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

        // Update the dock and control regions if the window size changes
        if size_changed {
            focus.send(Focus::ClaimRegion { program: our_program_id, region: vec![tool_dock.region_as_path(w, h)], z_index: DOCK_Z_INDEX }).await.ok();
        }

        // Redraw the dock if necessary
        if needs_redraw {
            let mut drawing = vec![];
            tool_dock.draw(&mut drawing, (w, h));

            context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}
