use crate::scenery::ui::*;
use super::sprite_manager::*;
use super::tool_state::*;
use super::tool_graphics::*;

use flo_binding::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_scene_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_curves::arc::*;

use futures::prelude::*;

use std::collections::*;
use std::sync::*;

/// Distance that a control is 'pulled' before it starts being dragged
const PULL_DISTANCE: f64    = 48.0;

const TOOL_WIDTH: f64       = 48.0;
const DOCK_Z_INDEX: usize   = 1000;

///
/// Representation of a tool in the floating tool dock
///
#[derive(PartialEq)]
struct FloatingTool {
    /// ID for this tool
    id: ToolId,

    /// Control ID for this tool
    control_id: ControlId,

    /// The name of this tool
    name: Binding<String>,

    /// Where the tool is anchored (its home position)
    anchor: Binding<(f64, f64)>,

    /// The position of this tool
    position: Binding<UiPoint>,

    /// The offset from the tool's current position due to the user dragging the tool
    drag_offset: Binding<Option<(f64, f64)>>,

    /// The instructions to draw the icon for this tool
    icon: Binding<Arc<Vec<Draw>>>,

    /// The sprite ID for this tool
    sprite: Binding<Option<SpriteId>>,

    // Update count for the sprite for this tool
    sprite_update: Binding<usize>,

    /// Where the tool has been dragged to (if it's been dragged)
    drag_position: Binding<Option<(f64, f64)>>,

    /// True if the dialog for this tool is open
    dialog_open: Binding<bool>,

    /// True if this tool is selected
    selected: Binding<bool>,

    /// True if the mouse is over this tool
    highlighted: Binding<bool>,

    /// True if this control has keyboard focus
    focused: Binding<bool>,

    /// True if the user is pressing on this tool
    pressed: Binding<bool>,

    /// True if the control is being dragged
    dragged: Binding<bool>,
}

///
/// State of the flaot
///
struct FloatingToolDock {
    program_id:     SubProgramId,
    tools:          Binding<Arc<HashMap<ToolId, Arc<FloatingTool>>>>,
    layer_id:       LayerId,
    namespace_id:   NamespaceId,
}

///
/// The floating tool dock manages tools that the user has dragged onto the background
///
pub async fn floating_tool_dock_program(input: InputStream<ToolState>, context: SceneContext, layer_id: LayerId) {
    let our_program_id = context.current_program_id().unwrap();

    let mut sprite_manager = context.send(()).unwrap();

    // Create the tool dock state
    let tool_dock = FloatingToolDock {
        program_id:     our_program_id,
        tools:          bind(Arc::new(HashMap::new())),
        layer_id:       layer_id,
        namespace_id:   *DOCK_LAYER,
    };
    let tool_dock = Arc::new(tool_dock);

    // Start the other subprograms that manage this tool dock
    let drawing_subprogram_id   = SubProgramId::new();
    let tool_dock_copy          = tool_dock.clone();
    context.send_message(SceneControl::start_child_program(drawing_subprogram_id, our_program_id, move |input, context| drawing_program(input, context, tool_dock_copy), 20)).await.ok();

    let events_subprogram_id    = SubProgramId::new();
    let tool_dock_copy          = tool_dock.clone();
    context.send_message(SceneControl::start_child_program(events_subprogram_id, our_program_id, move |input, context| events_program(input, context, tool_dock_copy), 20)).await.ok();

    let focus_subprogram_id = SubProgramId::new();
    let tool_dock_copy      = tool_dock.clone();
    context.send_message(SceneControl::start_child_program(focus_subprogram_id, our_program_id, move |input, context| focus_program(input, context, tool_dock_copy, events_subprogram_id), 20)).await.ok();

    // Start tracking tool state events
    let mut input = input;

    while let Some(input) = input.next().await {
        let tools = tool_dock.tools.get();

        match input {
            ToolState::AddTool(tool_id) => {
                // Create a new set of tools with the specified tool in it
                let mut tools   = (*tools).clone();
                let new_tool    = FloatingTool {
                    id:             tool_id,
                    control_id:     ControlId::new(),
                    name:           bind("".into()),
                    anchor:         bind((0.0, 0.0)),
                    position:       bind(UiPoint(0.0, 0.0)),
                    drag_offset:    bind(None),
                    icon:           bind(Arc::new(vec![])),
                    sprite:         bind(None),
                    sprite_update:  bind(0),
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),
                    focused:        bind(false),
                    pressed:        bind(false),
                    dragged:        bind(false),
                };
                tools.insert(tool_id, Arc::new(new_tool));

                tool_dock.tools.set(Arc::new(tools));
            },

            ToolState::DuplicateTool(duplicate_from, duplicate_to) => {
                // Fetch the tool to duplicate
                let Some(duplicate_from) = tools.get(&duplicate_from) else { continue; };

                // Create a duplicate of this tool (binding.clone() points to the same binding so we have to copy the bindings)
                let mut tools = (*tools).clone();
                let new_tool    = FloatingTool {
                    id:             duplicate_to,
                    control_id:     ControlId::new(),
                    name:           bind(duplicate_from.name.get()),
                    anchor:         bind(duplicate_from.anchor.get()),
                    position:       bind(duplicate_from.position.get()),
                    drag_offset:    bind(None),
                    icon:           bind(duplicate_from.icon.get()),
                    sprite:         bind(None),
                    sprite_update:  bind(0),
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),
                    focused:        bind(false),
                    pressed:        bind(false),
                    dragged:        bind(false),
                };
                tools.insert(duplicate_to, Arc::new(new_tool));

                // TODO: redraw the sprite when the tool is duplicated

                tool_dock.tools.set(Arc::new(tools));
            },

            ToolState::RemoveTool(tool_id) => {
                // Create a copy of the tools with the tool removed
                let mut tools = (*tools).clone();
                
                if let Some(old_tool) = tools.remove(&tool_id) {
                    if let Some(old_sprite) = old_tool.sprite.get() {
                        old_tool.sprite.set(None);
                        sprite_manager.send(SpriteManager::ReturnSprite(old_sprite)).await.ok();
                    }
                }
                
                tool_dock.tools.set(Arc::new(tools));
            },

            ToolState::Select(tool_id) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.selected.set(true);
            },

            ToolState::Deselect(tool_id) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.selected.set(false);
            },

            ToolState::LocateTool(tool_id, position) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.anchor.set(position);
                tool.position.set(UiPoint(position.0, position.1));
            },

            ToolState::SetName(tool_id, new_name) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.name.set(new_name);
            }

            ToolState::SetIcon(tool_id, drawing) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.icon.set(drawing.clone());

                // Assign a sprite if none is assigned to this tool
                let sprite_id = if let Some(sprite_id) = tool.sprite.get() {
                    sprite_id
                } else {
                    let sprite_id = assign_sprite(&context).await;

                    tool.sprite.set(Some(sprite_id));

                    sprite_id
                };

                // Draw the sprite
                let mut draw_sprite = vec![];

                draw_sprite.push_state();
                draw_sprite.namespace(tool_dock.namespace_id);
                draw_sprite.sprite(sprite_id);
                draw_sprite.clear_sprite();
                draw_sprite.extend(drawing.iter().cloned());
                draw_sprite.pop_state();

                context.send_message(DrawingRequest::Draw(Arc::new(draw_sprite))).await.ok();

                // Ensure that it's up to date in the render
                tool.sprite_update.set(tool.sprite_update.get() + 1);
            },

            ToolState::SetDialogLocation(_, _) => { },

            ToolState::OpenDialog(tool_id) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.dialog_open.set(true);
            },

            ToolState::CloseDialog(tool_id) => {
                let Some(tool) = tools.get(&tool_id) else { continue; };
                tool.dialog_open.set(false);
            },
        }
    }
}

///
/// Runs the program that draws the floating tools
///
async fn drawing_program(input: InputStream<BindingProgram>, context: SceneContext, floating_dock: Arc<FloatingToolDock>) {
    // The binding generates the drawing actions for the current scene
    let binding = computed(move || {
        let mut drawing = vec![];

        // Move to the layer
        drawing.push_state();
        drawing.namespace(floating_dock.namespace_id);
        drawing.layer(floating_dock.layer_id);
        drawing.clear_layer();

        // Draw each of the tools
        for (_, tool) in floating_dock.tools.get().iter() {
            let sprite_id       = tool.sprite.get();
            let UiPoint(x, y)   = tool.position.get();

            let (x, y) = if let Some((drag_x, drag_y)) = tool.drag_offset.get() {
                (x + drag_x, y + drag_y)
            } else {
                (x, y)
            };

            // Draw the plinth beneath the tool
            let plinth_x    = x - (TOOL_WIDTH / 2.0);
            let plinth_y    = y - (TOOL_WIDTH / 2.0);

            if tool.selected.get() {
                drawing.tool_plinth((plinth_x as _, plinth_y as _), (TOOL_WIDTH as _, TOOL_WIDTH as _), ToolPlinthState::FloatingSelected);
            } else if tool.pressed.get() {
                drawing.tool_plinth((plinth_x as _, plinth_y as _), (TOOL_WIDTH as _, TOOL_WIDTH as _), ToolPlinthState::FloatingPressed);
            } else if tool.highlighted.get() {
                drawing.tool_plinth((plinth_x as _, plinth_y as _), (TOOL_WIDTH as _, TOOL_WIDTH as _), ToolPlinthState::FloatingHighlighted);
            } else {
                drawing.tool_plinth((plinth_x as _, plinth_y as _), (TOOL_WIDTH as _, TOOL_WIDTH as _), ToolPlinthState::FloatingUnselected);
            }

            // Draw the sprite, if there is one
            if let Some(sprite_id) = sprite_id {
                drawing.push_state();
                tool.sprite_update.get();

                drawing.sprite_transform(SpriteTransform::Scale(1.2, 1.2));
                if tool.pressed.get() && tool.selected.get() {
                    drawing.sprite_transform(SpriteTransform::Translate(x as _, (y+6.0) as _));
                } else if tool.pressed.get() || tool.selected.get() {
                    drawing.sprite_transform(SpriteTransform::Translate(x as _, (y+3.0) as _));
                } else {
                    drawing.sprite_transform(SpriteTransform::Translate(x as _, y as _));
                }
                drawing.draw_sprite(sprite_id);
                drawing.pop_state();
            }
        }

        drawing.pop_state();

        drawing
    });

    // The action sends the drawing action to whatever subprogram is listening
    let action = BindingAction::new(|drawing: Vec<Draw>, context| async move {
        context.send_message(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
    });

    binding_program(input, context, binding, action).await;
}

///
/// Runs the program that generates the focus requests for the tools
///
async fn focus_program(input: InputStream<BindingProgram>, context: SceneContext, floating_dock: Arc<FloatingToolDock>, events_subprogram: SubProgramId) {
    // These track the positions of the tools
    let binding = computed(move || {
        let tools           = floating_dock.tools.get();
        let tool_positions  = tools.iter().map(|(tool_id, tool)| (*tool_id, tool.control_id, tool.position.get()));

        tool_positions.collect::<Vec<_>>()
    });

    // Action sets the positions of the various tools
    let existing_tools = Arc::new(Mutex::new(HashSet::new()));
    let action = BindingAction::new(move |positions: Vec<(ToolId, ControlId, UiPoint)>, context| {
        let existing_tools = existing_tools.clone();

        async move {
            let mut focus = context.send(()).unwrap();

            focus.send(Focus::ClaimRegion { program: events_subprogram, region: vec![], z_index: DOCK_Z_INDEX }).await.ok();

            // Tools that exist after this pass
            let mut still_existing_tools = HashSet::new();

            for (_tool_id, control_id, UiPoint(x, y)) in positions {
                // Claim the region for this tool
                let region = Circle::new(UiPoint(x, y), TOOL_WIDTH/2.0);
                let region = region.to_path::<UiPath>();

                focus.send(Focus::RemoveControlClaim(events_subprogram, control_id)).await.ok();
                focus.send(Focus::ClaimControlRegion { program: events_subprogram, region: vec![region], control: control_id, z_index: 0 }).await.ok();

                // Add the tool to the list that we know exists
                still_existing_tools.insert(control_id);
            }

            // Remove any tools that are no longer present
            let missing_tools = existing_tools.lock().unwrap().iter()
                .filter(|tool_id| !still_existing_tools.contains(tool_id))
                .copied()
                .collect::<Vec<_>>();

            for deleted_tool in missing_tools {
                focus.send(Focus::RemoveControlClaim(events_subprogram, deleted_tool)).await.ok();
            }

            // Replace the existing tools for the next pass through
            *(existing_tools.lock().unwrap()) = still_existing_tools;
        }
    });

    binding_program(input, context, binding, action).await;
}

///
/// Handles focus events for the floating tool dock
///
async fn events_program(input: InputStream<FocusEvent>, context: SceneContext, floating_dock: Arc<FloatingToolDock>) {
    let mut input = input;
    while let Some(evt) = input.next().await {
        // Default handling
        floating_dock.process_focus_event(&evt);

        // Looking for clicks and drags
        match evt {
            FocusEvent::Event(control_id, DrawEvent::Pointer(PointerAction::ButtonDown, pointer_id, pointer_state)) => {
                if pointer_state.buttons.contains(&Button::Left) {
                    // Fetch the tool that was clicked on
                    let tools       = floating_dock.tools.get();
                    let Some(tool)  = tools.iter().filter(|(_, tool)| Some(tool.control_id) == control_id).next() else { continue; };

                    track_left_down(&mut input, &context, floating_dock.clone(), tool.1.clone(), pointer_id, pointer_state).await;
                } else if pointer_state.buttons.contains(&Button::Right) {
                    // TODO: context menu (or maybe open the dialog?)
                }
            }

            _ => { }
        }
    }
}

///
/// Tracks the actions performed after the user has pressed the mouse down on a tool
///
async fn track_left_down(input: &mut InputStream<FocusEvent>, context: &SceneContext, floating_dock: Arc<FloatingToolDock>, tool: Arc<FloatingTool>, initial_pointer_id: PointerId, initial_state: PointerState) {
    tool.pressed.set(true);

    while let Some(evt) = input.next().await {
        // Standard behaviours still happen
        floating_dock.process_focus_event(&evt);

        // Track until the user releases the mouse or drags the tool
        match evt {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, pointer_id, pointer_state)) |
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Drag, pointer_id, pointer_state)) => {
                if pointer_id != initial_pointer_id { continue; }

                // Get the distance the user has dragged the cursor
                let Some((x1, y1))          = initial_state.location_in_canvas else { continue; };
                let Some((x2, y2))          = pointer_state.location_in_canvas else { continue; };
                let (offset_x, offset_y)    = ((x2-x1), (y2-y1));
                let distance                = (offset_x*offset_x + offset_y*offset_y).sqrt();

                // 'Pull' the tool away from its current position before entering the main drag
                let offset_ratio        = 1.0 - ((PULL_DISTANCE - distance) / PULL_DISTANCE);
                let offset_ratio        = offset_ratio.powi(2).min(1.0);
                let (pull_x, pull_y)    = (offset_x * offset_ratio, offset_y * offset_ratio);

                tool.drag_offset.set(Some((pull_x, pull_y)));

                if distance >= PULL_DISTANCE {
                    // Run the actual drag
                    track_left_drag(input, context, floating_dock.clone(), tool.clone(), initial_pointer_id, initial_state).await;
                    break;
                }
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, pointer_id, pointer_state)) => {
                if pointer_id != initial_pointer_id { continue; }
                if pointer_state.buttons.contains(&Button::Left) { continue; }

                // Select the tool
                context.send_message(Tool::Select(tool.id)).await.ok();

                break;
            }

            _ => { }
        }
    }

    tool.pressed.set(false);
    tool.dragged.set(false);
    tool.drag_offset.set(None);
}

///
/// Tracks the actions performed after the user has dragged a tool away from its current position
///
async fn track_left_drag(input: &mut InputStream<FocusEvent>, context: &SceneContext, floating_dock: Arc<FloatingToolDock>, tool: Arc<FloatingTool>, initial_pointer_id: PointerId, initial_state: PointerState) {
    tool.pressed.set(false);
    tool.dragged.set(true);

    while let Some(evt) = input.next().await {
        // Standard behaviours still happen
        floating_dock.process_focus_event(&evt);

        // Track until the user releases the mouse or drags the tool
        match evt {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, pointer_id, pointer_state)) |
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Drag, pointer_id, pointer_state)) => {
                if pointer_id != initial_pointer_id { continue; }

                // Get the distance the user has dragged the cursor
                let Some((x1, y1))          = initial_state.location_in_canvas else { continue; };
                let Some((x2, y2))          = pointer_state.location_in_canvas else { continue; };
                let (offset_x, offset_y)    = ((x2-x1), (y2-y1));

                tool.drag_offset.set(Some((offset_x, offset_y)));
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, pointer_id, pointer_state)) => {
                if pointer_id != initial_pointer_id { continue; }
                if pointer_state.buttons.contains(&Button::Left) { continue; }

                let Some((x1, y1))          = initial_state.location_in_canvas else { break; };
                let Some((x2, y2))          = pointer_state.location_in_canvas else { break; };
                let (offset_x, offset_y)    = ((x2-x1), (y2-y1));

                let UiPoint(cx, cy)         = tool.position.get();
                let (newx, newy)            = (cx + offset_x, cy + offset_y);

                // Move the tool to the new location
                context.send_message(Tool::SetToolLocation(tool.id, floating_dock.program_id.into(), (newx, newy))).await.ok();

                break;
            }

            _ => { }
        }
    }

    tool.pressed.set(false);
    tool.dragged.set(false);
    tool.drag_offset.set(None);
}

impl FloatingToolDock {
    ///
    /// Performs processing for the 'common' focus events which don't have any 'contextual' behaviour (as happens with drags, etc)
    ///
    fn process_focus_event(&self, evt: &FocusEvent) {
        match evt {
            FocusEvent::Event(_, DrawEvent::Resize(_, _)) => {
            }

            FocusEvent::Event(_, DrawEvent::Scale(_)) => {
            }

            FocusEvent::Focused(control_id) => {
                // Keyboard focus is on a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id == *control_id {
                            tool.focused.set(true);
                        }
                    });
            }

            FocusEvent::Unfocused(control_id) => {
                // Keyboard focus has left a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id == *control_id {
                            tool.focused.set(false);
                        }
                    });
            }

            FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Enter, _, _)) => {
                // Pointer has entered a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id == *control_id {
                            tool.highlighted.set(true);
                        }
                    });
            }

            FocusEvent::Event(Some(control_id), DrawEvent::Pointer(PointerAction::Leave, _, _)) => {
                // Pointer has left a tool
                self.tools.get().values()
                    .for_each(|tool| {
                        if tool.control_id == *control_id {
                            tool.highlighted.set(false);
                        }
                    });
            }

            _ => { }
        }
    }
}
