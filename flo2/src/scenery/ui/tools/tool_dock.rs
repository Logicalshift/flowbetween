//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use crate::scenery::ui::*;
use super::tool_state::*;
use super::tool_graphics::*;

use flo_curves::bezier::path::*;
use flo_scene::*;
use flo_binding::*;
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
const DOCK_SIDE_MARGIN: f64 = 4.0;
const DOCK_Z_INDEX: usize   = 1000;

///
/// Message sent to a tool dock
///
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Clone, PartialEq)]
struct ToolData {
    position:       Binding<(f64, f64)>,
    icon:           Binding<Arc<Vec<Draw>>>,
    sprite:         Binding<Option<SpriteId>>,
    control_id:     Binding<ControlId>,
    highlighted:    Binding<bool>,
    focused:        Binding<bool>,
    selected:       Binding<bool>,
    dialog_open:    Binding<bool>,
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
    tools:          Binding<Arc<HashMap<ToolId, ToolData>>>,
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
        let y = topleft.1 + DOCK_TOOL_GAP*3.0 + DOCK_TOOL_WIDTH / 2.0;

        // Draw the tools in order
        let mut y = y;
        for (_, tool) in self.ordered_tools() {
            tool.draw(gc, (x, y));

            y += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
        }
    }

    ///
    /// Returns the tools in order
    ///
    pub fn ordered_tools<'a>(&'a self) -> impl 'a + Iterator<Item=(ToolId, ToolData)> {
        let mut ordered_tools = self.tools.get().iter().map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>();
        ordered_tools.sort_by(|(_, a), (_, b)| a.position.get().1.total_cmp(&b.position.get().1));

        ordered_tools.into_iter()
    }

    ///
    /// Sets/updates the dialog position for a tool
    ///
    pub async fn set_dialog_position(&self, tool_state: &mut OutputSink<Tool>, tool_to_set: ToolId, w: f64, h: f64) {
        let (topleft, bottomright) = self.region(w, h);

        // Center point of the topmost tool
        let x = (topleft.0 + bottomright.0) / 2.0;
        let y = topleft.1 + DOCK_TOOL_GAP*3.0 + DOCK_TOOL_WIDTH / 2.0;

        // Order the tools by y-pos
        let ordered_tools = self.ordered_tools();

        // Draw the tools in order
        let mut y = y;
        for (tool_id, _) in ordered_tools {
            if tool_id == tool_to_set {
                tool_state.send(Tool::SetToolDialogLocation(tool_to_set, (x, y))).await.ok();
            }

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
        let state = if self.selected.get() {
            ToolPlinthState::Selected
        } else if self.highlighted.get() || self.focused.get() {
            ToolPlinthState::Highlighted
        } else {
            ToolPlinthState::Unselected
        };
        gc.tool_plinth(((center_pos.0 - DOCK_TOOL_WIDTH/2.0) as _, (center_pos.1 - DOCK_TOOL_WIDTH/2.0) as _), (DOCK_TOOL_WIDTH as _, DOCK_TOOL_WIDTH as _), state);

        // Draw the sprite for this tool
        if let Some(sprite_id) = self.sprite.get() {
            gc.push_state();
            gc.sprite_transform(SpriteTransform::Translate(center_pos.0 as _, center_pos.1 as _));
            gc.draw_sprite(sprite_id);
            gc.pop_state();
        }
    }

    ///
    /// The selection path for this tool when centered at the specified x and y positions
    ///
    pub fn outline_region(&self, x: f64, y: f64) -> UiPath {
        BezierPathBuilder::start(UiPoint(x - DOCK_TOOL_WIDTH/2.0, y - DOCK_TOOL_WIDTH/2.0))
            .line_to(UiPoint(x + DOCK_TOOL_WIDTH/2.0, y - DOCK_TOOL_WIDTH/2.0))
            .line_to(UiPoint(x + DOCK_TOOL_WIDTH/2.0, y + DOCK_TOOL_WIDTH/2.0))
            .line_to(UiPoint(x - DOCK_TOOL_WIDTH/2.0, y + DOCK_TOOL_WIDTH/2.0))
            .line_to(UiPoint(x - DOCK_TOOL_WIDTH/2.0, y - DOCK_TOOL_WIDTH/2.0))
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

    // The tool state subprogram manages which tools are selected at any given type
    let mut tool_state      = context.send(()).unwrap();
    let mut unused_sprites  = vec![];
    let mut next_sprite     = SpriteId(0);

    // Tool dock data
    let tool_dock = Arc::new(ToolDock {
        position:   position,
        layer:      layer,
        namespace:  *DOCK_LAYER,
        tools:      bind(Arc::new(HashMap::new())),
    });

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

                ToolDockMessage::FocusEvent(FocusEvent::Focused(control_id)) => {
                    // Keyboard focus is on a tool
                    tool_dock.tools.get().values()
                        .for_each(|tool| {
                            if tool.control_id.get() == control_id {
                                tool.focused.set(true);
                                needs_redraw = true;
                            }
                        });
                }

                ToolDockMessage::FocusEvent(FocusEvent::Unfocused(control_id)) => {
                    // Keyboard focus has left a tool
                    tool_dock.tools.get().values()
                        .for_each(|tool| {
                            if tool.control_id.get() == control_id {
                                tool.focused.set(false);
                                needs_redraw = true;
                            }
                        });
                }

                ToolDockMessage::FocusEvent(FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Enter, _, _))) => {
                    // Pointer has entered a tool
                    tool_dock.tools.get().values()
                        .for_each(|tool| {
                            if tool.control_id.get() == control_id {
                                tool.highlighted.set(true);
                                needs_redraw = true;
                            }
                        });
                }

                ToolDockMessage::FocusEvent(FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Leave, _, _))) => {
                    // Pointer has left a tool
                    tool_dock.tools.get().values()
                        .for_each(|tool| {
                            if tool.control_id.get() == control_id {
                                tool.highlighted.set(false);
                                needs_redraw        = true;
                            }
                        });
                }

                ToolDockMessage::FocusEvent(FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::ButtonDown, _, _))) => {
                    let tools = tool_dock.tools.get();

                    // User has clicked on a tool
                    let selected_tool = tools.iter()
                        .filter(|(_, tool)| tool.control_id.get() == control_id)
                        .next();

                    if let Some((tool_id, _)) = selected_tool {
                        let tool_id = *tool_id;

                        // Toggle the tool's dialog if the user clicks the tool that's already selected
                        if let Some(tool) = tools.get(&tool_id) {
                            if tool.selected.get() && !tool.dialog_open.get() {
                                tool.dialog_open.set(true);

                                tool_dock.set_dialog_position(&mut tool_state, tool_id, w, h).await;
                                tool_state.send(Tool::OpenDialog(tool_id)).await.ok();
                            } else {
                                tool.dialog_open.set(false);
                                tool_state.send(Tool::CloseDialog(tool_id)).await.ok();
                            }
                        }

                        // Select this tool
                        tool_state.send(Tool::Select(tool_id)).await.ok();
                    }
                }

                ToolDockMessage::ToolState(ToolState::AddTool(tool_id)) => { 
                    // Add (or replace) the tool with this ID
                    let mut new_tools = (*tool_dock.tools.get()).clone();

                    new_tools.insert(tool_id, ToolData {
                        position:       bind((0.0, 0.0)),
                        icon:           bind(Arc::new(vec![])),
                        sprite:         bind(None),
                        control_id:     bind(ControlId::new()),
                        selected:       bind(false),
                        highlighted:    bind(false),
                        focused:        bind(false),
                        dialog_open:    bind(false),
                    });

                    tool_dock.tools.set(Arc::new(new_tools));

                    // Draw with the new tool
                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::SetIcon(tool_id, icon)) => {
                    // Update the icon
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.icon.set(icon);

                        if let Some(old_sprite) = tool.sprite.get() {
                            tool.sprite.set(None);

                            // Remove the sprite to force the tool to redraw its icon
                            unused_sprites.push(old_sprite);
                        }
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::LocateTool(tool_id, position)) => {
                    // Change the position (we use the y position to set the ordering in the dock)0
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.position.set(position);
                    }

                    needs_redraw = true;
                    size_changed = true;
                }

                ToolDockMessage::ToolState(ToolState::RemoveTool(tool_id)) => {
                    // Remove the tool from this dock
                    let mut new_tools = (*tool_dock.tools.get()).clone();

                    if let Some(old_tool) = new_tools.remove(&tool_id) {
                        if let Some(old_sprite) = old_tool.sprite.get() {
                            old_tool.sprite.set(None);
                            unused_sprites.push(old_sprite);
                        }

                        focus.send(Focus::RemoveControlClaim(our_program_id, old_tool.control_id.get())).await.ok();
                    }

                    needs_redraw = true;
                    size_changed = true;
                }

                ToolDockMessage::ToolState(ToolState::Select(tool_id)) => {
                    // Mark this tool as selected
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.selected.set(true);
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::Deselect(tool_id)) => {
                    // Mark this tool as unselected
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.selected.set(false);
                    }

                    needs_redraw = true;
                }

                ToolDockMessage::ToolState(ToolState::OpenDialog(tool_id)) => {
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.dialog_open.set(true);
                    }
                }

                ToolDockMessage::ToolState(ToolState::CloseDialog(tool_id)) => {
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.dialog_open.set(false);
                    }
                }

                ToolDockMessage::ToolState(_)  => { /* Other toolstate messages are ignored */ }
                ToolDockMessage::FocusEvent(_) => { /* Other focus events are ignored */ }
            }
        }

        // Update the dock and control regions if the window size changes
        if size_changed {
            // Claim the overall region
            focus.send(Focus::ClaimRegion { program: our_program_id, region: vec![tool_dock.region_as_path(w, h)], z_index: DOCK_Z_INDEX }).await.ok();

            // Claim the position of each tool
            let (topleft, bottomright) = tool_dock.region(w, h);

            // Center point of the topmost tool
            let x = (topleft.0 + bottomright.0) / 2.0;
            let y = topleft.1 + DOCK_TOOL_GAP*3.0 + DOCK_TOOL_WIDTH / 2.0;

            let mut y = y;
            let mut z = 0;
            for (_, tool) in tool_dock.ordered_tools() {
                let region = tool.outline_region(x, y);
                focus.send(Focus::ClaimControlRegion { program: our_program_id, control: tool.control_id.get(), region: vec![region], z_index: z }).await.ok();

                y += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
                z += 1;
            }
        }

        // Redraw the dock if necessary
        if needs_redraw {
            let mut drawing = vec![];

            // Write out the sprites for the tools
            let tools           = tool_dock.tools.get();

            for (_, tool_data) in tools.iter() {
                if tool_data.sprite.get().is_none() {
                    // Assign a sprite ID (either re-use one we've used before or assign a new one)
                    let sprite_id = if let Some(sprite_id) = unused_sprites.pop() {
                        sprite_id
                    } else {
                        let sprite_id = next_sprite;
                        next_sprite = SpriteId(next_sprite.0+1);

                        sprite_id
                    };

                    tool_data.sprite.set(Some(sprite_id));

                    // Draw the tool to create the sprite
                    drawing.push_state();
                    drawing.namespace(tool_dock.namespace);
                    drawing.sprite(sprite_id);
                    drawing.clear_sprite();

                    drawing.extend(tool_data.icon.get().iter().cloned());

                    drawing.pop_state();
                }
            }

            // Draw the tool dock
            tool_dock.draw(&mut drawing, (w, h));

            context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}
