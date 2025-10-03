//!
//! The tool_state subprogram manages the overall state of where tools are located in a document. The state
//! can be monitored or queried in order to draw the tools, or updated to set which tools are available
//! in the application
//!

use crate::scenery::ui::subprograms::*;

use flo_scene::*;
use flo_draw::canvas::*;

use futures::prelude::*;
use ::serde::*;
use uuid::*;

use std::collections::*;
use std::sync::*;

///
/// Identifier used to specify a tool group within the flowbetween app. One tool can be selected
/// per tool group.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolGroupId(Uuid);

impl ToolGroupId {
    ///
    /// Creates a unique new tool group ID
    ///
    pub fn new() -> Self {
        ToolGroupId(Uuid::new_v4())
    }
}

///
/// Identifier for a type of tool. Eg, there might be a tool type indicating 'colour'.
///
/// In FlowBetween, tools can be duplicated: eg, you might have multiple colour tools that
/// can be used to instantly switch between colours. The tool type is used so that these tools
/// can be identified.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolTypeId(Uuid);

impl ToolTypeId {
    pub fn new() -> Self {
        ToolTypeId(Uuid::new_v4())
    }
}


///
/// Identifier for an individual tool
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolId(Uuid);

impl ToolId {
    pub fn new() -> Self {
        ToolId(Uuid::new_v4())
    }
}

///
/// Message type that performs actions relating to the tool state
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Tool {
    /// Adds a subscriber to the general tool status events
    Subscribe(StreamTarget),

    /// Queries the general tools status
    Query(StreamTarget),

    /// Creates a new tool (which is initially not positioned anywhere)
    CreateTool(ToolGroupId, ToolTypeId, ToolId),

    /// Removes an existing tool from the state
    RemoveTool(ToolId),

    /// Creates a duplicate of the first tool ID, where the new tool has the next tool ID
    /// The subprogram that runs the tool is expected to 
    DuplicateTool(ToolId, ToolId),

    /// Sets the 'owner' subprogram for a type of tool (this is responsible for displaying dialogs and managing settings for the tool and also dealing with what happens when a tool is selected)
    SetToolOwner(ToolTypeId, StreamTarget),

    /// Sets the subprogram that represents the 'location' of the tool. This program is responsible for rendering it and sending events when the tool is selected.
    ///
    /// Different subprograms can arrange tools and send selection events how they like. For example, a dock program might use the y position to order the tools,
    /// but a 'floating' program might allow tools to be placed anywhere.
    SetToolLocation(ToolId, StreamTarget, (f64, f64)),

    /// Sets the location where the tool's configuration dialog should be displayed if opened
    SetToolDialogLocation((f64, f64)),

    /// Sets the name of a tool
    SetToolName(ToolId, String),

    /// Sets the drawing instructions to render the icon for a tool (tool's location is 0,0 relative to the drawing)
    SetToolIcon(ToolId, Arc<Vec<Draw>>),

    /// Sets which tool is selected by default for a particular group (used when the selected tool is removed to prevent all tools from being removed)
    SetDefaultForGroup(ToolGroupId, ToolId),

    /// Sets a tool as a selected (unselecting any other tools in the group)
    Select(ToolId),

    /// Opens the configuration dialog for a tool ID
    OpenDialog(ToolId),

    /// Close any open dialogs
    CloseDialogs,
}

impl SceneMessage for Tool {
    fn default_target() -> StreamTarget {
        subprogram_tool_state().into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Run the tool state subprogram
        init_context.connect_programs((), subprogram_tool_state(), StreamId::with_message_type::<Tool>()).unwrap();
        init_context.add_subprogram(subprogram_tool_state(), tool_state_program, 20);
    }

}

///
/// Messages sent to the subprogram that is the location of a tool
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ToolLocation {

}

impl SceneMessage for ToolLocation {
    fn default_target() -> StreamTarget {
        StreamTarget::None
    }
}

///
/// Messages sent to the subprogram that is set as the owner of a type of tool
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ToolOwner {

}

impl SceneMessage for ToolOwner {
    fn default_target() -> StreamTarget {
        StreamTarget::None
    }
}

///
/// Messages sent to anything that queries or subscribes to the state of the tools in FlowBetween
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ToolState {
    /// The specified tool has become selected
    Select(ToolId),

    /// The specified tool has become deselected
    Deselect(ToolId),
}

impl SceneMessage for ToolState {
    fn default_target() -> StreamTarget {
        StreamTarget::None
    }
}

///
/// The default subprogram that manages tool state (which tools are selected, where their icons are drawn) in a flowbetween document
///
pub async fn tool_state_program(input: InputStream<Tool>, context: SceneContext) {
    // The values that make up the known state of the tools in FlowBetween
    let mut tools_for_groups    = HashMap::new();
    let mut group_for_tool      = HashMap::new();
    let mut type_for_tool       = HashMap::new();
    let mut tools               = HashSet::new();

    let mut tool_locations      = HashMap::new();
    let mut tool_type_owners    = HashMap::new();
    let mut tool_names          = HashMap::new();
    let mut tool_icons          = HashMap::new();

    let mut group_selection     = HashMap::new();

    let mut subscribers         = vec![];

    // Sends a message to all the Subscribers
    async fn send_to_subscribers(subscribers: &mut Vec<Option<OutputSink<ToolState>>>, message: ToolState) {
        // Send the message to each subscriber in turn
        for maybe_subscriber in subscribers.iter_mut() {
            if let Some(subscriber) = maybe_subscriber {
                let status = subscriber.send(message.clone()).await;

                if let Err(_) = status {
                    // Remove this subscriber
                    *maybe_subscriber = None;
                }
            }
        }

        // Free up any
        subscribers.retain(|val| val.is_some());
    }

    // Run the main loop
    let mut input = input;
    while let Some(tool_request) = input.next().await {
        use Tool::*;

        match tool_request {
            Subscribe(subscribe_target) => {
                if let Ok(mut subscription_target) = context.send::<ToolState>(subscribe_target) {
                    // Send the current state
                    for selected_tool in group_selection.values().copied() { subscription_target.send(ToolState::Select(selected_tool)).await.ok(); }
                    
                    // Add to the subscription list
                    subscribers.push(Some(subscription_target))
                }
            }

            Query(query_target) => {
                // TODO: send the current state as a query response
                todo!();
            }

            SetToolOwner(tool_type_id, tool_owner_target) => {
                if let Some(owner) = tool_type_owners.get_mut(&tool_type_id) {
                    *owner = context.send::<ToolOwner>(tool_owner_target).ok();
                }
            }

            SetToolLocation(tool_id, location_target, location) => {
                if let Some(location) = tool_locations.get_mut(&tool_id) {
                    *location = context.send::<ToolLocation>(location_target).ok();
                }
            }

            SetToolDialogLocation(location) => {
                todo!();
            }

            SetToolName(tool_id, new_name) => {
                if let Some(name) = tool_names.get_mut(&tool_id) {
                    *name = new_name;
                }
            }

            SetToolIcon(tool_id, new_drawing) => {
                if let Some(icon) = tool_icons.get_mut(&tool_id) {
                    *icon = new_drawing;
                }
            }

            CreateTool(group_id, type_id, tool_id) => {
                // Add this tool to the group
                tools_for_groups.entry(group_id)
                    .or_insert_with(|| HashSet::new())
                    .insert(tool_id);
                group_for_tool.insert(tool_id, group_id);
                type_for_tool.insert(tool_id, type_id);
                tools.insert(tool_id);

                // Set up the default properties
                tool_locations.insert(tool_id, None);
                tool_names.insert(tool_id, String::new());
                tool_type_owners.entry(type_id).or_insert(None);
                tool_icons.insert(tool_id, Arc::new(vec![]));
            }

            RemoveTool(tool_id) => {
                todo!();
            }

            DuplicateTool(old_tool_id, new_tool_id) => {
                todo!();
            }

            SetDefaultForGroup(tool_group_id, default_tool_id) => {
                todo!();
            }

            Select(tool_id) => {
                if let Some(tool_group) = group_for_tool.get(&tool_id) {
                    if let Some(old_tool) = group_selection.get(tool_group) {
                        // Deselect the old tool in the state
                        send_to_subscribers(&mut subscribers, ToolState::Deselect(*old_tool)).await;

                        // TODO: deselect in the location of the tool
                        // TODO: deselect in the owner of the tool
                    }

                    // Select the new tool
                    group_selection.insert(*tool_group, tool_id);

                    // Send to subscribers the new tool
                    send_to_subscribers(&mut subscribers, ToolState::Select(tool_id)).await;

                    // TODO: send to the tool's location that it's now selected
                    // TODO: send to the tool's owner that it's now selected
                }
            }

            OpenDialog(tool_id) => {
                todo!();
            }

            CloseDialogs => {
                todo!();
            }

        }
    }
}
