//!
//! A tool dock contains a fixed set of tools and allows user to select one per tool group.
//!

use crate::scenery::ui::*;
use super::sprite_manager::*;
use super::tool_state::*;
use super::tool_graphics::*;

use flo_curves::bezier::path::*;
use flo_scene::programs::*;
use flo_scene::*;
use flo_scene_binding::*;
use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;

use futures::prelude::*;

use std::collections::*;
use std::f64;
use std::sync::*;

/// Distance that a control is 'pulled' before it starts being dragged
const PULL_DISTANCE: f64    = 64.0;

const DOCK_WIDTH: f64       = 48.0;
const DOCK_TOOL_WIDTH: f64  = 38.0;
const DOCK_TOOL_GAP: f64    = 2.0;
const DOCK_TOP_MARGIN: f64  = 100.0;
const DOCK_SIDE_MARGIN: f64 = 4.0;
const DOCK_Z_INDEX: usize   = 1000;

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
    tool_id:        ToolId,
    position:       Binding<(f64, f64)>,
    icon:           Binding<Arc<Vec<Draw>>>,
    sprite:         Binding<Option<SpriteId>>,
    sprite_update:  Binding<usize>,
    control_id:     Binding<ControlId>,
    highlighted:    Binding<bool>,
    pressed:        Binding<bool>,
    focused:        Binding<bool>,
    selected:       Binding<bool>,
    dialog_open:    Binding<bool>,

    center:         Binding<(f64, f64)>,

    drag_fade:      Binding<f64>,
    drag_position:  Binding<Option<(f64, f64)>>,
    drop_anim:      AnimationBinding,
    drop_cancel:    AnimationBinding,
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

    /// Size of the window (incorporating the scale)
    window_size:    Binding<(f64, f64)>,

    /// Scale of the window
    scale:          Binding<f64>,
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
        let mut ypos        = y;
        let ordered_tools   = self.ordered_tools();

        for (_, tool) in ordered_tools.iter() {
            tool.draw(gc, (x, ypos));

            ypos += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
        }

        let mut ypos = y;
        for (_, tool) in ordered_tools.iter() {
            tool.draw_overlay(gc, (x, ypos));

            ypos += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
        }
    }

    ///
    /// Returns the tools in order
    ///
    pub fn ordered_tools(&self) -> Vec<(ToolId, ToolData)> {
        let mut ordered_tools = self.tools.get().iter().map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>();
        ordered_tools.sort_by(|(_, a), (_, b)| a.position.get().1.total_cmp(&b.position.get().1));

        ordered_tools
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
        // When we're binding, we want to redraw this tool if the sprite update count changes
        self.sprite_update.get();

        // Draw the 'plinth' for this tool
        let state = if self.selected.get() {
            ToolPlinthState::Selected
        } else if self.pressed.get() {
            ToolPlinthState::Pressed
        } else if self.highlighted.get() || self.focused.get() {
            ToolPlinthState::Highlighted
        } else {
            ToolPlinthState::Unselected
        };
        gc.tool_plinth(((center_pos.0 - DOCK_TOOL_WIDTH/2.0) as _, (center_pos.1 - DOCK_TOOL_WIDTH/2.0) as _), (DOCK_TOOL_WIDTH as _, DOCK_TOOL_WIDTH as _), state);

        // Draw the sprite for this tool
        if let Some(sprite_id) = self.sprite.get() {
            gc.push_state();

            if self.pressed.get() {
                gc.sprite_transform(SpriteTransform::Translate(center_pos.0 as _, (center_pos.1+2.0) as _));
            } else {
                gc.sprite_transform(SpriteTransform::Translate(center_pos.0 as _, center_pos.1 as _));
            }

            if self.dialog_open.get() {
                gc.new_path();
                gc.circle((center_pos.0 + DOCK_TOOL_WIDTH/2.0 - 6.0) as _, (center_pos.1 + DOCK_TOOL_WIDTH/2.0 - 6.0) as _, 3.0);
                gc.fill_color(color_tool_dock_outline());
                gc.fill();
            }

            gc.draw_sprite(sprite_id);
            gc.pop_state();
        }
    }

    ///
    /// Second pass of drawing: anything that should be rendered above all the other tools
    ///
    pub fn draw_overlay(&self, gc: &mut impl GraphicsContext, _center_pos: (f64, f64)) {
        if let Some(sprite_id) = self.sprite.get() {
            // If the tool is being dragged, draw a second copy at that position
            if let Some((drag_x, drag_y)) = self.drag_position.get() {
                let drag_fade       = self.drag_fade.get();
                let mut drag_scale  = 1.0 + (drag_fade * 0.35);
                let drop_anim       = self.drop_anim.get();
                let drop_cancel     = self.drop_cancel.get();

                if drop_anim > 0.0 {
                    let basic_scale     = 1.35 - (0.15 * drop_anim);
                    let wobble          = -(drop_anim * 2.0 * f64::consts::PI).sin();
                    let wobble_factor   = 0.3 - (0.1 * drop_anim);

                    drag_scale = basic_scale + (wobble * wobble_factor);
                }

                if drop_cancel > 0.0 {
                    drag_scale *= 1.0 - drop_cancel;
                }

                gc.push_state();

                gc.tool_plinth(((drag_x - DOCK_TOOL_WIDTH/2.0*drag_scale) as _, (drag_y - DOCK_TOOL_WIDTH/2.0*drag_scale) as _), ((DOCK_TOOL_WIDTH*drag_scale) as _, (DOCK_TOOL_WIDTH*drag_scale) as _), ToolPlinthState::StartDrag(drag_fade));

                if drop_anim > 0.0 {
                    let radius = (1.35*DOCK_TOOL_WIDTH/2.0) + (DOCK_TOOL_WIDTH/3.0)*drop_anim;
                    gc.new_path();
                    gc.circle(drag_x as _, drag_y as _, radius as _);
                    gc.line_width(1.0);
                    gc.stroke_color(color_tool_border().with_alpha((1.0-drop_anim) as _));
                    gc.stroke();
                }

                gc.sprite_transform(SpriteTransform::Scale(drag_scale as _, drag_scale as _));
                gc.sprite_transform(SpriteTransform::Translate(drag_x as _, drag_y as _));
                gc.draw_sprite(sprite_id);
                gc.pop_state();
            }
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
pub async fn tool_dock_program(input: InputStream<ToolState>, context: SceneContext, position: DockPosition, layer: LayerId, floating_tools_program: Option<SubProgramId>) {
    let our_program_id = context.current_program_id().unwrap();

    // The focus subprogram is used to send events to the dock
    let mut focus           = context.send(()).unwrap();
    let mut sprite_manager  = context.send(()).unwrap();

    // Tool dock data
    let tool_dock = Arc::new(ToolDock {
        position:       position,
        layer:          layer,
        namespace:      *DOCK_LAYER,
        tools:          bind(Arc::new(HashMap::new())),
        window_size:    bind((1000.0, 1000.0)),
        scale:          bind(1.0),
    });

    // Run the child program that handles redrawing the tool dock
    let drawing_subprogram  = SubProgramId::new();
    let tool_dock_copy      = tool_dock.clone();

    context.send_message(SceneControl::start_child_program(drawing_subprogram, our_program_id, move |input, context| tool_dock_drawing_program(input, context, tool_dock_copy), 10)).await.ok();

    // Run the child program that handles events for this tool dock
    let events_subprogram   = SubProgramId::new();
    let tool_dock_copy      = tool_dock.clone();

    context.send_message(SceneControl::start_child_program(events_subprogram, our_program_id, move |input, context| tool_dock_focus_events_program(input, context, tool_dock_copy, floating_tools_program), 10)).await.ok();

    // Run the child program that deals with resizing the dock (placing focus areas, mainly)
    let resizing_subprogram = SubProgramId::new();
    let tool_dock_copy      = tool_dock.clone();

    context.send_message(SceneControl::start_child_program(resizing_subprogram, our_program_id, move |input, context| tool_dock_resizing_program(input, context, tool_dock_copy, events_subprogram), 10)).await.ok();

    // Run the program
    let mut input = input.ready_chunks(50);
    while let Some(msgs) = input.next().await {
        // Process the messages that are waiting
        for msg in msgs {
            match msg {
                ToolState::AddTool(tool_id) => { 
                    // Add (or replace) the tool with this ID
                    let mut new_tools = (*tool_dock.tools.get()).clone();

                    new_tools.insert(tool_id, ToolData {
                        tool_id:        tool_id,
                        position:       bind((0.0, 0.0)),
                        icon:           bind(Arc::new(vec![])),
                        sprite:         bind(None),
                        sprite_update:  bind(0),
                        control_id:     bind(ControlId::new()),
                        selected:       bind(false),
                        highlighted:    bind(false),
                        pressed:        bind(false),
                        focused:        bind(false),
                        dialog_open:    bind(false),

                        center:         bind((0.0, 0.0)),

                        drag_fade:      bind(0.0),
                        drag_position:  bind(None),
                        drop_anim:      animate_binding(AnimationDescription::default(), &context),
                        drop_cancel:    animate_binding(AnimationDescription::default(), &context),
                    });

                    tool_dock.tools.set(Arc::new(new_tools));
                }

                ToolState::DuplicateTool(original_tool_id, duplicate_tool_id) => {
                    // Create a duplicate of the specified tool
                    let mut new_tools   = (*tool_dock.tools.get()).clone();
                    let Some(old_tool)  = new_tools.get(&original_tool_id) else { continue; };

                    new_tools.insert(duplicate_tool_id, ToolData {
                        tool_id:        duplicate_tool_id,
                        position:       bind(old_tool.position.get()),
                        icon:           bind(old_tool.icon.get()),
                        sprite:         bind(None),
                        sprite_update:  bind(0),
                        control_id:     bind(ControlId::new()),
                        selected:       bind(false),
                        highlighted:    bind(false),
                        pressed:        bind(false),
                        focused:        bind(false),
                        dialog_open:    bind(false),

                        center:         bind((0.0, 0.0)),

                        drag_fade:      bind(0.0),
                        drag_position:  bind(None),
                        drop_anim:      animate_binding(AnimationDescription::default(), &context),
                        drop_cancel:    animate_binding(AnimationDescription::default(), &context),
                    });

                    tool_dock.tools.set(Arc::new(new_tools));
                },

                ToolState::SetIcon(tool_id, icon) => {
                    // Update the icon
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        // Set the icon value
                        tool.icon.set(icon);

                        // Update the sprite for this tool
                        let sprite_id = if let Some(sprite_id) = tool.sprite.get() {
                            sprite_id
                        } else {
                            // Assign a sprite ID (either re-use one we've used before or assign a new one)
                            let sprite_id = assign_sprite(&context).await;

                            tool.sprite.set(Some(sprite_id));

                            sprite_id
                        };

                        // Draw the tool to create the sprite
                        let mut drawing = vec![];

                        drawing.push_state();
                        drawing.namespace(tool_dock.namespace);
                        drawing.sprite(sprite_id);
                        drawing.clear_sprite();

                        drawing.extend(tool.icon.get().iter().cloned());

                        drawing.pop_state();

                        context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();

                        // Cause a redraw by updating the sprite update count
                        tool.sprite_update.set(tool.sprite_update.get() + 1);
                    }
                }

                ToolState::LocateTool(tool_id, position) => {
                    // Change the position (we use the y position to set the ordering in the dock)0
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.position.set(position);
                    }
                }

                ToolState::RemoveTool(tool_id) => {
                    // Remove the tool from this dock
                    let mut new_tools = (*tool_dock.tools.get()).clone();

                    if let Some(old_tool) = new_tools.remove(&tool_id) {
                        if let Some(old_sprite) = old_tool.sprite.get() {
                            old_tool.sprite.set(None);
                            sprite_manager.send(SpriteManager::ReturnSprite(old_sprite)).await.ok();
                        }

                        focus.send(Focus::RemoveControlClaim(events_subprogram, old_tool.control_id.get())).await.ok();

                        tool_dock.tools.set(Arc::new(new_tools));
                    }
                }

                ToolState::Select(tool_id) => {
                    // Mark this tool as selected
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.selected.set(true);
                    }
                }

                ToolState::Deselect(tool_id) => {
                    // Mark this tool as unselected
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.selected.set(false);
                    }
                }

                ToolState::OpenDialog(tool_id) => {
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.dialog_open.set(true);
                    }
                }

                ToolState::CloseDialog(tool_id) => {
                    if let Some(tool) = tool_dock.tools.get().get(&tool_id) {
                        tool.dialog_open.set(false);
                    }
                }

                ToolState::SetName(_, _)            => { },
                ToolState::SetDialogLocation(_, _)  => { },
            }
        }
    }
}

///
/// A child subprogram that draws the tool dock
///
async fn tool_dock_drawing_program(input: InputStream<BindingProgram>, context: SceneContext, tool_dock: Arc<ToolDock>) {
    let namespace   = tool_dock.namespace;
    let layer_id    = tool_dock.layer;

    // Binding action just draws the layer and clears it out when the program finishes
    let drawing_action = BindingAction::new(move |drawing_actions: Vec<Draw>, context| async move {
        context.send_message(DrawingRequest::Draw(Arc::new(drawing_actions))).await.ok();
    }).with_stop_action(move |context| async move {
        context.send_message(DrawingRequest::Draw(Arc::new(vec![
            Draw::PushState,

            Draw::Namespace(namespace),
            Draw::Layer(layer_id),
            Draw::ClearLayer,

            Draw::PopState,
        ]))).await.ok();
    });

    // The drawing binding converts the tool dock into a set of drawing actions
    let drawing_binding = computed(move || {
        let mut drawing = vec![];
        tool_dock.draw(&mut drawing, tool_dock.window_size.get());
        drawing
    });

    // Start a binding program
    binding_program(input, context, drawing_binding, drawing_action).await;
}

///
/// A child subprogram that deals with resizing the tool dock and repositioning any tools that it contains
///
async fn tool_dock_resizing_program(input: InputStream<BindingProgram>, context: SceneContext, tool_dock: Arc<ToolDock>, events_subprogram: SubProgramId) {
    #[derive(Clone, PartialEq)]
    struct BindingData {
        window_size:    (f64, f64),
        ordered_tools:  Vec<(ToolId, ToolData)>,
        region:         (UiPoint, UiPoint),
        region_as_path: UiPath,
    }

    // Binding is the values we need to perform the resizing
    let size_binding = computed(move || {
        let (w, h)          = tool_dock.window_size.get();
        let ordered_tools   = tool_dock.ordered_tools();
        let region          = tool_dock.region(w, h);
        let region_as_path  = tool_dock.region_as_path(w, h);

        BindingData {
            window_size:    (w, h),
            ordered_tools:  ordered_tools,
            region:         region,
            region_as_path: region_as_path
        }
    });

    // Action is to reposition the tools
    let resize_action = BindingAction::new(move |data: BindingData, context| async move {
        let mut focus = context.send(()).unwrap();

        let ordered_tools   = data.ordered_tools;
        let region_as_path  = data.region_as_path;
        let region          = data.region;

        // Claim the overall region
        focus.send(Focus::ClaimRegion { program: events_subprogram, region: vec![region_as_path], z_index: DOCK_Z_INDEX }).await.ok();

        // Claim the position of each tool
        let (topleft, bottomright) = region;

        // Center point of the topmost tool
        let x = (topleft.0 + bottomright.0) / 2.0;
        let y = topleft.1 + DOCK_TOOL_GAP*3.0 + DOCK_TOOL_WIDTH / 2.0;

        let mut y = y;
        let mut z = 0;
        for (_, tool) in ordered_tools {
            let region = tool.outline_region(x, y);
            focus.send(Focus::ClaimControlRegion { program: events_subprogram, control: tool.control_id.get(), region: vec![region], z_index: z }).await.ok();

            tool.center.set((x, y));

            y += DOCK_TOOL_WIDTH + DOCK_TOOL_GAP;
            z += 1;
        }
    });

    // Start a binding program
    binding_program(input, context, size_binding, resize_action).await;
}

impl ToolDock {
    ///
    /// Performs processing for the 'common' focus events which don't have any 'contextual' behaviour (as happens with drags, etc)
    ///
    fn process_focus_event(&self, evt: &FocusEvent) {
        match evt {
            FocusEvent::Event(_, DrawEvent::Resize(new_w, new_h)) => {
                // Update the width and height
                let scale   = self.scale.get();
                let w       = new_w / scale;
                let h       = new_h / scale;

                self.window_size.set((w, h));
            }

            FocusEvent::Event(_, DrawEvent::Scale(new_scale)) => {
                // Update the scale, adjust the width and height accordingly
                let (w, h)  = self.window_size.get();
                let scale   = self.scale.get();
                let w       = (w * scale) / new_scale;
                let h       = (h * scale) / new_scale;
                
                self.scale.set(*new_scale);
                self.window_size.set((w, h));
            }

            FocusEvent::Focused(control_id) => {
                // Keyboard focus is on a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id.get() == *control_id {
                            tool.focused.set(true);
                        }
                    });
            }

            FocusEvent::Unfocused(control_id) => {
                // Keyboard focus has left a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id.get() == *control_id {
                            tool.focused.set(false);
                        }
                    });
            }

            FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Enter, _, _)) => {
                // Pointer has entered a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id.get() == *control_id {
                            tool.highlighted.set(true);
                        }
                    });
            }

            FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Leave, _, _)) => {
                // Pointer has left a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id.get() == *control_id {
                            tool.highlighted.set(false);
                        }
                    });
            }

            _ => { }
        }
    }
}

///
/// A child subprogram that handles events for the tool dock
///
async fn tool_dock_focus_events_program(input: InputStream<FocusEvent>, context: SceneContext, tool_dock: Arc<ToolDock>, floating_tools_program: Option<SubProgramId>) {
    let our_program_id = context.current_program_id().unwrap();

    // The focus program deals with redirecting events to us
    let mut focus = context.send(()).unwrap();

    // Claim an initial region (this is so the focus subprogram sends us a greeting)
    {
        let (w, h) = tool_dock.window_size.get();
        focus.send(Focus::ClaimRegion { program: our_program_id, region: vec![tool_dock.region_as_path(w, h)], z_index: DOCK_Z_INDEX }).await.ok();
    }

    // Process as many events as possible each iteration
    let mut input = input;

    while let Some(msg) = input.next().await {
        // Standard event processing
        tool_dock.process_focus_event(&msg);

        // Pointer action event processing
        match msg {
            FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::ButtonDown, pointer_id, pointer_state)) => {
                let tools = tool_dock.tools.get();

                // User has clicked on a tool
                let selected_tool = tools.iter()
                    .filter(|(_, tool)| tool.control_id.get() == control_id)
                    .next();

                if let Some((tool_id, _)) = selected_tool {
                    let tool_id = *tool_id;

                    if let Some(tool) = tools.get(&tool_id) {
                        // Track this tool
                        track_button_down(&mut input, &context, pointer_state, &tool_dock, tool.clone(), pointer_id, floating_tools_program).await;
                    }
                }
            }

            _ => { }
        }
    }
}

///
/// The user has pressed a button over a tool: track as they move with the button pressed
///
async fn track_button_down(input: &mut InputStream<FocusEvent>, context: &SceneContext, initial_state: PointerState, tool_dock: &Arc<ToolDock>, clicked_tool: ToolData, pointer_id: PointerId, floating_tools_program: Option<SubProgramId>) {
    let Some(initial_pos) = initial_state.location_in_canvas else { return; };

    // Set the tool as pressed
    clicked_tool.pressed.set(true);

    let mut pulling = false;

    // Track events until the user releases the button
    while let Some(msg) = input.next().await {
        // Default processing happens as normal
        tool_dock.process_focus_event(&msg);

        // Track until the user releases the mouse button
        match msg {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, evt_pointer_id, pointer_state)) |
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Drag, evt_pointer_id, pointer_state)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                // 'Pull' this item (or start dragging it)
                let Some(drag_pos)          = pointer_state.location_in_canvas else { continue; };
                let (offset_x, offset_y)    = (drag_pos.0 - initial_pos.0, drag_pos.1 - initial_pos.1);
                let distance                = ((offset_x*offset_x) + (offset_y*offset_y)).sqrt();

                if !pulling && distance <= 4.0 {
                    // Tool has to be dragged a certain distance before we start 'pulling' it
                } else if distance <= PULL_DISTANCE {
                    pulling = true;

                    // Pull the control (increasing force pulling it back the closer it is to its home position)
                    let offset_ratio        = 1.0 - ((PULL_DISTANCE - distance) / PULL_DISTANCE);
                    let offset_ratio        = offset_ratio.powi(2);
                    let (pull_x, pull_y)    = (offset_x * offset_ratio, offset_y * offset_ratio);
                    let (cx, cy)            = clicked_tool.center.get();

                    clicked_tool.drag_fade.set(offset_ratio);
                    clicked_tool.drag_position.set(Some((cx + pull_x, cy + pull_y)));
                } else {
                    // Drag the control
                    track_button_drag(input, context, initial_state, tool_dock, clicked_tool.clone(), pointer_id, floating_tools_program).await;
                    return;
                }
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, evt_pointer_id, pointer_state)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                let Some((x, y))    = pointer_state.location_in_canvas else { break; };
                let (cx, cy)        = clicked_tool.center.get();
                if x >= (cx-DOCK_TOOL_WIDTH/2.0) && y >= (cy-DOCK_TOOL_WIDTH/2.0) && x <= (cx+DOCK_TOOL_WIDTH/2.0) && y <= (cy+DOCK_TOOL_WIDTH/2.0) {
                    if clicked_tool.selected.get() {
                        // Toggle the dialog for the tool if it's already selected
                        if !clicked_tool.dialog_open.get() {
                            let mut tool_state  = context.send(()).unwrap();
                            let (w, h)          = tool_dock.window_size.get();

                            tool_dock.set_dialog_position(&mut tool_state, clicked_tool.tool_id, w, h).await;
                            tool_state.send(Tool::OpenDialog(clicked_tool.tool_id)).await.ok();
                       } else {
                            context.send(()).unwrap().send(Tool::CloseDialog(clicked_tool.tool_id)).await.ok();
                        }
                    } else {
                        // Select this tool when the mouse is released and is still over it
                        context.send(()).unwrap().send(Tool::Select(clicked_tool.tool_id)).await.ok();
                    }
                }

                // Finished
                break;
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Cancel, evt_pointer_id, _)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                // Finished
                break;
            }

            _ => { }
        }
    }

    // Clear the pressed status
    clicked_tool.pressed.set(false);
    clicked_tool.drag_position.set(None);
    clicked_tool.drop_anim.reset();
    clicked_tool.drop_cancel.reset();
}

///
/// The user has pressed a button over a tool: track as they move with the button pressed
///
async fn track_button_drag(input: &mut InputStream<FocusEvent>, _context: &SceneContext, initial_state: PointerState, tool_dock: &Arc<ToolDock>, clicked_tool: ToolData, pointer_id: PointerId, floating_tools_program: Option<SubProgramId>) {
    let Some(initial_pos) = initial_state.location_in_canvas else { return; };

    // Unpress the tool once it starts dragging
    clicked_tool.pressed.set(false);
    clicked_tool.drop_anim.reset();
    clicked_tool.drop_cancel.reset();

    // Track events until the user releases the button
    while let Some(msg) = input.next().await {
        // Default processing happens as normal
        tool_dock.process_focus_event(&msg);

        // Track until the user releases the mouse button
        match msg {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, evt_pointer_id, pointer_state)) |
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Drag, evt_pointer_id, pointer_state)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                let Some(drag_pos)  = pointer_state.location_in_canvas else { continue; };
                let (x, y)          = drag_pos;

                // If the tool is within the region for this dock, then it fades out
                let (w, h) = tool_dock.window_size.get();
                let region = tool_dock.region(w, h);

                let UiPoint(x1, y1) = region.0;
                let UiPoint(x2, y2) = region.1;

                let drag_fade = if x >= x1 && x <= x2 && y >= y1 && y <= y2 {
                    let offset_to_center = (((x1+x2)/2.0) - x).abs();
                    (offset_to_center/(DOCK_WIDTH/2.0)).max(0.0).min(1.0)*0.5 + 0.5
                } else {
                    1.0
                };

                // Drag this item to the new position
                let (offset_x, offset_y)    = (drag_pos.0 - initial_pos.0, drag_pos.1 - initial_pos.1);
                let (cx, cy)                = clicked_tool.center.get();

                clicked_tool.drag_fade.set(drag_fade);
                clicked_tool.drag_position.set(Some((cx + offset_x, cy + offset_y)));
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, evt_pointer_id, pointer_state)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                // Check if the tool has been dropped back down on the dock
                let (x, y) = if let Some(pos) = pointer_state.location_in_canvas { pos } else { (0.0, 0.0) };
                let (w, h) = tool_dock.window_size.get();
                let region = tool_dock.region(w, h);

                let UiPoint(x1, y1) = region.0;
                let UiPoint(x2, y2) = region.1;

                // Get the position that the tool has been dragged to
                let (offset_x, offset_y)    = (x - initial_pos.0, y - initial_pos.1);
                let (cx, cy)                = clicked_tool.center.get();

                clicked_tool.drag_position.set(Some((cx + offset_x, cy + offset_y)));

                let animated_tool = clicked_tool.clone();
                if (x >= x1 && x <= x2 && y >= y1 && y <= y2) || floating_tools_program.is_none() {
                    // On the dock: draw a 'failing' animation
                    clicked_tool.drop_cancel.change_animation(AnimationDescription::ease_in(0.15)
                        .with_when_finished(move |_| async move {
                            // Unset the animation
                            animated_tool.pressed.set(false);
                            animated_tool.drag_position.set(None);
                            animated_tool.drop_anim.reset();
                            animated_tool.drop_cancel.reset();
                        }));
                    clicked_tool.drop_cancel.start();
                } else {
                    // Drop this tool in its new position when the tool is released
                    let floating_tools_program  = floating_tools_program.unwrap();
                    clicked_tool.drop_anim.change_animation(AnimationDescription::linear(0.4)
                        .with_when_finished(move |context| async move {
                            // Finish the drop by updating the tool state
                            let new_tool_id = ToolId::new();
                            context.send_message(Tool::DuplicateTool(clicked_tool.tool_id, new_tool_id)).await.ok();
                            context.send_message(Tool::SetToolLocation(new_tool_id, floating_tools_program.into(), (cx + offset_x, cy + offset_y))).await.ok();

                            context.wait_for_idle(100).await;

                            // Unset the animation
                            animated_tool.pressed.set(false);
                            animated_tool.drag_position.set(None);
                            animated_tool.drop_anim.reset();
                            animated_tool.drop_cancel.reset();

                        }));
                    clicked_tool.drop_anim.start();
                }

                // Finished
                return;
            }

            FocusEvent::Event(_, DrawEvent::KeyDown(_, Some(Key::KeyEscape))) => {
                // Cancel the drag if escape is pressed
                let animated_tool = clicked_tool.clone();
                clicked_tool.drop_cancel.change_animation(AnimationDescription::ease_in(0.15)
                    .with_when_finished(move |_| async move {
                        // Unset the animation
                        animated_tool.pressed.set(false);
                        animated_tool.drag_position.set(None);
                        animated_tool.drop_anim.reset();
                        animated_tool.drop_cancel.reset();
                    }));
                clicked_tool.drop_cancel.start();

                // Finished
                return;
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Cancel, evt_pointer_id, _)) => {
                // Ignore events from other pointers
                if evt_pointer_id != pointer_id { continue; }

                let animated_tool = clicked_tool.clone();
                clicked_tool.drop_cancel.change_animation(AnimationDescription::ease_in(0.15)
                    .with_when_finished(move |_| async move {
                        // Unset the animation
                        animated_tool.pressed.set(false);
                        animated_tool.drag_position.set(None);
                        animated_tool.drop_anim.reset();
                        animated_tool.drop_cancel.reset();
                    }));
                clicked_tool.drop_cancel.start();

                // Finished
                return;
            }

            _ => { }
        }
    }

    // Unpress/drag the tool
    clicked_tool.pressed.set(false);
    clicked_tool.drag_position.set(None);
    clicked_tool.drop_anim.reset();
    clicked_tool.drop_cancel.reset();
}
