use crate::scenery::ui::*;
use super::sprite_manager::*;
use super::tool_state::*;
use super::tool_graphics::*;

use flo_binding::*;
use flo_scene::*;
use flo_scene::commands::*;
use flo_scene::programs::*;
use flo_scene_binding::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;

use futures::prelude::*;

use std::collections::*;
use std::sync::*;

const TOOL_WIDTH: f64 = 48.0;

///
/// Representation of a tool in the floating tool dock
///
#[derive(PartialEq)]
struct FloatingTool {
    /// ID for this tool
    id: ToolId,

    /// The name of this tool
    name: Binding<String>,

    /// Where the tool is anchored (its home position)
    anchor: Binding<(f64, f64)>,

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

    /// True if the user is pressing on this tool
    pressed: Binding<bool>,
}

///
/// State of the flaot
///
struct FloatingToolDock {
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
        tools:          bind(Arc::new(HashMap::new())),
        layer_id:       layer_id,
        namespace_id:   *DOCK_LAYER,
    };
    let tool_dock = Arc::new(tool_dock);

    // Start the other subprograms that manage this tool dock
    let drawing_subprogram = SubProgramId::new();

    let tool_dock_copy = tool_dock.clone();
    context.send_message(SceneControl::start_child_program(drawing_subprogram, our_program_id, move |input, context| drawing_program(input, context, tool_dock_copy), 20)).await.ok();

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
                    name:           bind("".into()),
                    anchor:         bind((0.0, 0.0)),
                    icon:           bind(Arc::new(vec![])),
                    sprite:         bind(None),
                    sprite_update:  bind(0),
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),
                    pressed:        bind(false),
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
                    name:           bind(duplicate_from.name.get()),
                    anchor:         bind(duplicate_from.anchor.get()),
                    icon:           bind(duplicate_from.icon.get()),
                    sprite:         bind(None),
                    sprite_update:  bind(0),
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),
                    pressed:        bind(false),
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
                    let mut sprite_id               = context.spawn_query(ReadCommand::default(), Query::<AssignedSprite>::with_no_target(), ()).unwrap();
                    let AssignedSprite(sprite_id)   = sprite_id.next().await.unwrap();

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
            let sprite_id   = tool.sprite.get();
            let (x, y)      = tool.anchor.get();

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
                tool.sprite_update.get();

                // drawing.sprite_transform(SpriteTransform::Scale(1.2, 1.2));
                if tool.pressed.get() || tool.selected.get() {
                    drawing.sprite_transform(SpriteTransform::Translate(x as _, (y+3.0) as _));
                } else {
                    drawing.sprite_transform(SpriteTransform::Translate(x as _, y as _));
                }
                drawing.draw_sprite(sprite_id);
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
