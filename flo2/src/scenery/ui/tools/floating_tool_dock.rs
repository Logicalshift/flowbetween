use super::tool_state::*;

use flo_binding::*;
use flo_scene::*;
use flo_draw::canvas::*;

use futures::prelude::*;

use std::collections::*;
use std::sync::*;

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

    /// Where the tool has been dragged to (if it's been dragged)
    drag_position: Binding<Option<(f64, f64)>>,

    /// True if the dialog for this tool is open
    dialog_open: Binding<bool>,

    /// True if this tool is selected
    selected: Binding<bool>,

    /// True if the mouse is over this tool
    highlighted: Binding<bool>,
}

///
/// State of the flaot
///
struct FloatingToolDock {
    tools:      Binding<Arc<HashMap<ToolId, Arc<FloatingTool>>>>,
    layer_id:   LayerId,
}

///
/// The floating tool dock manages tools that the user has dragged onto the background
///
pub async fn floating_tool_dock_program(input: InputStream<ToolState>, context: SceneContext, layer_id: LayerId) {
    // Create the tool dock state
    let tool_dock = FloatingToolDock {
        tools:      bind(Arc::new(HashMap::new())),
        layer_id:   layer_id,
    };
    let tool_dock = Arc::new(tool_dock);

    // Start the other subprograms that manage this tool dock

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
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),

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
                    sprite:         bind(duplicate_from.sprite.get()),
                    drag_position:  bind(None),
                    dialog_open:    bind(false),
                    selected:       bind(false),
                    highlighted:    bind(false),

                };
                tools.insert(duplicate_to, Arc::new(new_tool));

                tool_dock.tools.set(Arc::new(tools));
            },

            ToolState::RemoveTool(tool_id) => {
                // Create a copy of the tools with the tool removed
                let mut tools = (*tools).clone();
                tools.remove(&tool_id);
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
                tool.icon.set(drawing);
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
