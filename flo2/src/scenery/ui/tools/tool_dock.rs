//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use crate::scenery::ui::*;
use super::tool_state::*;
use super::tool_graphics::*;

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
const DOCK_TOOL_WIDTH: f64  = 38.0;
const DOCK_TOOL_GAP: f64    = 2.0;
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
    FocusEvent(FocusEvent),
}

impl SceneMessage for ToolDockMessage {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|tool_state_msgs| tool_state_msgs.map(|msg| ToolDockMessage::ToolState(msg)))), (), StreamId::with_message_type::<ToolState>())
            .unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|draw_event_msgs| draw_event_msgs.map(|msg| ToolDockMessage::FocusEvent(msg)))), (), StreamId::with_message_type::<FocusEvent>())
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
    position:       (f64, f64),
    icon:           Arc<Vec<Draw>>,
    sprite:         Option<SpriteId>,
    highlighted:    bool,
    selected:       bool,
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

        // Draw the dock background
        let (topleft, bottomright) = self.region(window_size.0, window_size.1);
        gc.tool_dock((topleft.0 as _, topleft.1 as _), ((bottomright.0-topleft.0) as _, (bottomright.1-topleft.1) as _));

        // Draw the tools themselves
        self.draw_tools(gc, window_size);

        // Finish up by clearing the state
        gc.pop_state();
    }

    ///
    /// Draws the tools in this dock
    ///
    pub fn draw_tools(&self, gc: &mut impl GraphicsContext, window_size: (f64, f64)) {
        let (topleft, bottomright) = self.region(window_size.0, window_size.1);

        // Center point of the topmost tool
        let x = (topleft.0 + bottomright.0) / 2.0;
        let y = topleft.1 + DOCK_TOOL_GAP*2.0 + DOCK_TOOL_WIDTH / 2.0;

        // Order the tools by y-pos
        let mut ordered_tools = self.tools.values().collect::<Vec<_>>();
        ordered_tools.sort_by(|a, b| a.position.1.total_cmp(&b.position.1));

        // Draw the tools in order
        let mut y = y;
        for tool in ordered_tools {
            tool.draw(gc, (x, y));

            y += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
        }
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

impl ToolData {
    ///
    /// Draws this tool at the specified position
    ///
    pub fn draw(&self, gc: &mut impl GraphicsContext, center_pos: (f64, f64)) {
        // Draw the 'plinth' for this tool
        let state = if self.selected {
            ToolPlinthState::Selected
        } else if self.highlighted {
            ToolPlinthState::Highlighted
        } else {
            ToolPlinthState::Unselected
        };
        gc.tool_plinth(((center_pos.0 - DOCK_TOOL_WIDTH/2.0) as _, (center_pos.1 - DOCK_TOOL_WIDTH/2.0) as _), (DOCK_TOOL_WIDTH as _, DOCK_TOOL_WIDTH as _), state);

        // Draw the sprite for this tool
        if let Some(sprite_id) = self.sprite {
            gc.push_state();
            gc.sprite_transform(SpriteTransform::Translate(center_pos.0 as _, center_pos.1 as _));
            gc.draw_sprite(sprite_id);
            gc.pop_state();
        }
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
                ToolDockMessage::FocusEvent(FocusEvent::Event(_, DrawEvent::Resize(new_w, new_h))) => {
                    // Update the width and height
                    w = new_w / scale;
                    h = new_h / scale;

                    needs_redraw = true;
                    size_changed = true;
                }

                ToolDockMessage::FocusEvent(FocusEvent::Event(_, DrawEvent::Scale(new_scale))) => {
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
                        position:       (0.0, 0.0),
                        icon:           Arc::new(vec![]),
                        sprite:         None,
                        selected:       false,
                        highlighted:    false,
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

                ToolDockMessage::ToolState(_)  => { /* Other toolstate messages are ignored */ }
                ToolDockMessage::FocusEvent(_) => { /* Other focus events are ignored */ }
            }
        }

        // Update the dock and control regions if the window size changes
        if size_changed {
            focus.send(Focus::ClaimRegion { program: our_program_id, region: vec![tool_dock.region_as_path(w, h)], z_index: DOCK_Z_INDEX }).await.ok();
        }

        // Redraw the dock if necessary
        if needs_redraw {
            let mut drawing = vec![];

            // Write out the sprites for the tools
            let tools           = &mut tool_dock.tools;
            let next_sprite     = &mut tool_dock.next_sprite;
            let unused_sprites  = &mut tool_dock.unused_sprites;

            for (_, tool_data) in tools.iter_mut() {
                if tool_data.sprite.is_none() {
                    // Assign a sprite ID (either re-use one we've used before or assign a new one)
                    let sprite_id = if let Some(sprite_id) = unused_sprites.pop() {
                        sprite_id
                    } else {
                        let sprite_id = *next_sprite;
                        *next_sprite = SpriteId(next_sprite.0+1);

                        sprite_id
                    };

                    tool_data.sprite = Some(sprite_id);

                    // Draw the tool to create the sprite
                    drawing.push_state();
                    drawing.namespace(tool_dock.namespace);
                    drawing.sprite(sprite_id);
                    drawing.clear_sprite();

                    drawing.extend(tool_data.icon.iter().cloned());

                    drawing.pop_state();
                }
            }

            // Draw the tool dock
            tool_dock.draw(&mut drawing, (w, h));

            context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}
