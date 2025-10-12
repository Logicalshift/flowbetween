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
use std::iter;
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

    /// Creates a new tool (which is initially not positioned anywhere)
    CreateTool(ToolGroupId, ToolTypeId, ToolId),

    /// Removes an existing tool from the state
    RemoveTool(ToolId),

    /// Creates a duplicate of the first tool ID, where the new tool has the next tool ID
    /// The subprogram that runs the tool is expected to 
    DuplicateTool(ToolId, ToolId),

    ///
    /// Joins two tools such that they both become selected at the same time. Both tools should be in different tool
    /// groups, or this will have no effect.
    ///
    /// Both tools are commonly in the same location. The two tools have slightly different behaviour. The first tool
    /// is considered the 'main' tool. When the main tool is selected, all the other tools in the group are also selected.
    /// When it's deselected all of the other tools in the group are reverted to their values before the group was 
    /// selected. The second tool can be selected and deselected independently, leaving the group partially selected.
    ///
    /// If any of the secondary tools are deselected and the main tool is reselected, all of the secondary tools are
    /// reselected.
    ///
    /// In general, JoinTools should be called such that there's only one 'main' tool in any collections.
    ///
    /// This is intended to be used with a UI element that lets you select, say, a brush and its colour and/or layer
    /// all in a single action, using the 'DuplicateTool' functionality to create extra tools.
    ///
    JoinTools(ToolId, ToolId),

    ///
    /// If the specified tool is part of a group created by 'JoinTools', this will disconnect it from the group, so that
    /// it's selected indepedently again.
    ///
    DisconnectTool(ToolId),

    /// Sets the 'owner' subprogram for a type of tool (this is responsible for displaying dialogs and managing settings for the tool and also dealing with what happens when a tool is selected)
    SetToolTypeOwner(ToolTypeId, StreamTarget),

    /// Sets the subprogram that represents the 'location' of the tool. This program is responsible for rendering it and sending events when the tool is selected.
    ///
    /// Different subprograms can arrange tools and send selection events how they like. For example, a dock program might use the y position to order the tools,
    /// but a 'floating' program might allow tools to be placed anywhere.
    SetToolLocation(ToolId, StreamTarget, (f64, f64)),

    /// Sets the location where a tool's configuration dialog should be displayed if opened
    ///
    /// This is sent as feedback from the tool's location program, to indicate where the tool's dialog should be displayed when it's opened
    SetToolDialogLocation(ToolId, (f64, f64)),

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

    /// Closes the configuration dialog for a tool ID
    CloseDialog(ToolId),

    /// Close any open dialogs
    CloseAllDialogs,
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
/// Messages sent to anything that queries or subscribes to the state of the tools in FlowBetween
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ToolState {
    /// A new tool has been created
    AddTool(ToolId),

    /// Copies a tool to another tool
    DuplicateTool(ToolId, ToolId),

    /// A tool has been removed
    RemoveTool(ToolId),

    /// The specified tool has become selected
    Select(ToolId),

    /// The specified tool has become deselected
    Deselect(ToolId),

    /// Sets the location of a tool
    LocateTool(ToolId, (f64, f64)),

    /// Sets the name for a tool
    SetName(ToolId, String),

    /// Sets the icon for a tool
    SetIcon(ToolId, Arc<Vec<Draw>>),

    /// Sets the location on the canvas where the tool dialog should be opened around
    /// (This is the center of the tool, so the dialog should be created in a way that aligns with this location)
    SetDialogLocation(ToolId, (f64, f64)),

    /// Opens the dialog for configuring the specified tool
    OpenDialog(ToolId),

    /// Closes the dialog for configuring the specified tool
    CloseDialog(ToolId),
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
    let mut tools_for_groups        = HashMap::new();                   // List of tools in each group
    let mut tools_for_type          = HashMap::<_, HashSet<_>>::new();  // List of tools for each tool type
    let mut group_for_tool          = HashMap::new();                   // Maps tool IDs to their group (only one tool can be selected per group)
    let mut type_for_tool           = HashMap::new();                   // Maps tool IDs to their type (tool types determine which subprogram owns the tool)
    let mut tools                   = HashSet::new();                   // List of valid tool IDs
    let mut dialog_location         = HashMap::new();                   // The location coordinates set for each tool

    let mut joined_tools            = HashMap::<_, Vec<_>>::new();      // Tools that should be selected alongside another tool
    let mut join_parent_tool        = HashMap::new();                   // The 'parent' tool that each tool is joined to

    let mut tool_locations          = HashMap::new();                   // Stream for sending messages to the tool's current location
    let mut tool_location_targets   = HashMap::new();                   // StreamTarget corresponding to each tool_location
    let mut tool_type_owners        = HashMap::new();                   // The streams where messages for the owner of each tool type should be sent
    let mut tool_names              = HashMap::new();                   // The names set for each tool
    let mut tool_icons              = HashMap::new();                   // The drawing instructions to create the icons for each tool

    let mut group_selection         = HashMap::new();                   // The selected tool for each group
    let mut default_for_group       = HashMap::new();                   // The default selection for each group (used when a selected tool is deleted)

    let mut subscribers             = vec![];                           // Subscribers for general messages
    let mut open_dialogs            = HashSet::new();                   // Tools with open configuration dialogs

    // Sends a message to all the Subscribers
    async fn send_to_subscribers(subscribers: Option<&mut Vec<Option<OutputSink<ToolState>>>>, message: ToolState) {
        if let Some(subscribers) = subscribers {
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

            SetToolTypeOwner(tool_type_id, tool_owner_target) => {
                // Get or create the list of owners for this tool type
                let owner = tool_type_owners.entry(tool_type_id)
                    .or_insert_with(|| vec![]);

                // Update the owner
                *owner = vec![context.send::<ToolState>(tool_owner_target).ok()];

                // Add all of the tools of this type
                if let Some(tools) = tools_for_type.get(&tool_type_id) {
                    for tool_id in tools {
                        send_to_subscribers(Some(owner), ToolState::AddTool(*tool_id)).await;

                        if let Some(location) = dialog_location.get(tool_id) {
                            send_to_subscribers(Some(owner), ToolState::SetDialogLocation(*tool_id, *location)).await;
                        }
                    }
                }
            }

            SetToolIcon(tool_id, new_drawing) => {
                if let Some(icon) = tool_icons.get_mut(&tool_id) {
                    *icon = new_drawing.clone();

                    send_to_subscribers(tool_locations.get_mut(&tool_id), ToolState::SetIcon(tool_id, new_drawing)).await;
                }
            }

            SetToolLocation(tool_id, location_target, position) => {
                if let Some(location) = tool_locations.get_mut(&tool_id) {
                    // If the location target has changed, remove the tool from the original
                    let mut target_changed = false;
                    if let Some(old_target) = tool_location_targets.get(&tool_id) {
                        if old_target != &location_target {
                            target_changed = true;
                            send_to_subscribers(Some(location), ToolState::RemoveTool(tool_id)).await;
                        }
                    } else {
                        target_changed = true;
                    }

                    // Update the location
                    *location = vec![context.send::<ToolState>(location_target.clone()).ok()];
                    tool_location_targets.insert(tool_id, location_target);

                    // Add the tool in its new location
                    if target_changed {
                        send_to_subscribers(Some(location), ToolState::AddTool(tool_id)).await;

                        if let Some(icon) = tool_icons.get(&tool_id) {
                            send_to_subscribers(Some(location), ToolState::SetIcon(tool_id, icon.clone())).await;
                        }
                    }

                    // Set the location of the tool
                    send_to_subscribers(Some(location), ToolState::LocateTool(tool_id, position)).await;
                }
            }

            SetToolDialogLocation(tool_id, location) => {
                // Dialog location is for the tool owner (which is assumed to handle the tool's configuration dialog)
                send_to_subscribers(type_for_tool.get(&tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::SetDialogLocation(tool_id, location)).await;

                dialog_location.insert(tool_id, location);
            }

            SetToolName(tool_id, new_name) => {
                if let Some(name) = tool_names.get_mut(&tool_id) {
                    *name = new_name.clone();

                    send_to_subscribers(tool_locations.get_mut(&tool_id), ToolState::SetName(tool_id, new_name.clone())).await;
                    send_to_subscribers(type_for_tool.get(&tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::SetName(tool_id, new_name.clone())).await;
                    send_to_subscribers(Some(&mut subscribers), ToolState::SetName(tool_id, new_name.clone())).await;
                }
            }

            CreateTool(group_id, type_id, tool_id) => {
                // Add this tool to the group
                tools_for_groups.entry(group_id)
                    .or_insert_with(|| HashSet::new())
                    .insert(tool_id);
                tools_for_type.entry(type_id)
                    .or_insert_with(|| HashSet::new())
                    .insert(tool_id);
                group_for_tool.insert(tool_id, group_id);
                type_for_tool.insert(tool_id, type_id);
                tools.insert(tool_id);

                // Set up the default properties
                tool_locations.insert(tool_id, vec![]);
                tool_names.insert(tool_id, String::new());
                tool_type_owners.entry(type_id).or_insert(vec![]);
                tool_icons.insert(tool_id, Arc::new(vec![]));

                // Indicate that this tool has been added to the subscribers and the type owner
                send_to_subscribers(Some(&mut subscribers), ToolState::AddTool(tool_id)).await;
                send_to_subscribers(tool_type_owners.get_mut(&type_id), ToolState::AddTool(tool_id)).await;
            }

            RemoveTool(tool_id) => {
                // Remove from the basic types
                let tool_type   = type_for_tool.remove(&tool_id);
                let tool_group  = group_for_tool.remove(&tool_id);
                tool_names.remove(&tool_id);
                tool_icons.remove(&tool_id);
                tools.remove(&tool_id);
                tool_locations.remove(&tool_id);

                if let Some(parent_tool) = join_parent_tool.remove(&tool_id) {
                    if let Some(joined_with) = joined_tools.get_mut(&parent_tool) {
                        joined_with.retain(|old_tool_id| old_tool_id != &tool_id);

                        if joined_with.is_empty() {
                            joined_tools.remove(&parent_tool);
                        }
                    }
                }

                if let Some(joined_with) = joined_tools.remove(&tool_id) {
                    joined_with.into_iter()
                        .for_each(|join_tool_id| { join_parent_tool.remove(&join_tool_id); });
                }

                // Remove from the groups and types
                if let Some(tool_group) = tool_group.and_then(|group| tools_for_groups.get_mut(&group)) { tool_group.remove(&tool_id); }
                if let Some(tool_type) = tool_type.and_then(|tool_type| tools_for_type.get_mut(&tool_type)) { tool_type.remove(&tool_id); }

                // Remove from the default if this is the default tool
                if tool_group.and_then(|tool_group| default_for_group.get(&tool_group)) == Some(&tool_id) {
                    if let Some(tool_group) = tool_group { default_for_group.remove(&tool_group); }
                }

                // Tell the location that the tool is removed
                let mut old_location = tool_locations.remove(&tool_id);

                send_to_subscribers(old_location.as_mut(), ToolState::RemoveTool(tool_id)).await;
                tool_location_targets.remove(&tool_id);

                // Tell the owner that the tool is removed
                send_to_subscribers(tool_type.and_then(|tool_type| tool_type_owners.get_mut(&tool_type)), ToolState::RemoveTool(tool_id)).await;

                // Tell the other subscribers that the tool is removed
                send_to_subscribers(Some(&mut subscribers), ToolState::RemoveTool(tool_id)).await;

                // If this tool is selected for the group, then reselect the default tool
                if let (Some(tool_group), Some(selection)) = (tool_group, tool_group.and_then(|tool_group| group_selection.get(&tool_group))) {
                    if selection == &tool_id {
                        // Remove the tool from the group
                        group_selection.remove(&tool_group);

                        // Select the default tool if there is one (we say that removing the tool is the same as deselecting it so we only need to send the select message)
                        if let Some(default_tool) = default_for_group.get(&tool_group) {
                            send_to_subscribers(Some(&mut subscribers), ToolState::Select(*default_tool)).await;
                            send_to_subscribers(old_location.as_mut(), ToolState::Select(*default_tool)).await;
                            send_to_subscribers(tool_type.and_then(|tool_type| tool_type_owners.get_mut(&tool_type)), ToolState::Select(*default_tool)).await;
                        }
                    }
                }

                // Dialog will no longer be open for removed tools
                open_dialogs.remove(&tool_id);
            }

            DuplicateTool(old_tool_id, new_tool_id) => {
                // Get the properties for the new tool
                let tool_type   = type_for_tool.get(&old_tool_id).copied();
                let tool_group  = group_for_tool.get(&old_tool_id).copied();
                let tool_name   = tool_names.get(&old_tool_id).cloned();
                let tool_icon   = tool_icons.get(&old_tool_id).cloned();

                if let (Some(tool_type), Some(tool_group), Some(tool_name), Some(tool_icon)) = (tool_type, tool_group, tool_name, tool_icon) {
                    // Store the new tool
                    tools_for_groups.entry(tool_group).or_insert_with(|| HashSet::new()).insert(new_tool_id);
                    tools_for_type.entry(tool_type).or_insert_with(|| HashSet::new()).insert(new_tool_id);
                    group_for_tool.insert(new_tool_id, tool_group);
                    type_for_tool.insert(new_tool_id, tool_type);
                    tools.insert(new_tool_id);

                    tool_locations.insert(new_tool_id, vec![]);
                    tool_names.insert(new_tool_id, tool_name.clone());
                    tool_type_owners.entry(tool_type).or_insert(vec![]);
                    tool_icons.insert(new_tool_id, tool_icon.clone());

                    // Duplicate for the owners
                    send_to_subscribers(tool_type_owners.get_mut(&tool_type), ToolState::DuplicateTool(old_tool_id, new_tool_id)).await;

                    // Add for the location
                    if let Some(location_target) = tool_location_targets.get_mut(&old_tool_id) {
                        if let Some(location) = context.send(location_target.clone()).ok() {
                            let mut location = vec![Some(location)];

                            send_to_subscribers(Some(&mut location), ToolState::DuplicateTool(old_tool_id, new_tool_id)).await;
                            send_to_subscribers(Some(&mut location), ToolState::SetIcon(new_tool_id, tool_icon.clone())).await;

                            tool_locations.insert(new_tool_id, location);
                        }
                    }

                    // Indicate that the tool is added to the subscribers
                    send_to_subscribers(Some(&mut subscribers), ToolState::AddTool(new_tool_id)).await;
                }
            }

            JoinTools(main_tool, joined_tool) => {
                if tools.contains(&main_tool) && tools.contains(&joined_tool) {
                    // If the joined tool was already joined somewhere, remove that join
                    if let Some(previously_joined) = join_parent_tool.remove(&joined_tool) {
                        if let Some(joined_with) = joined_tools.get_mut(&previously_joined) {
                            joined_with.retain(|old_tool_id| old_tool_id != &joined_tool);

                            if joined_with.is_empty() {
                                joined_tools.remove(&previously_joined);
                            }
                        }
                    }

                    // Add as a joined tool
                    joined_tools.entry(main_tool)
                        .or_insert_with(|| vec![])
                        .push(joined_tool);
                    join_parent_tool.insert(joined_tool, main_tool);
                }
            }

            DisconnectTool(tool_id) => {
                // If the joined tool was already joined somewhere, remove that join
                if let Some(previously_joined) = join_parent_tool.remove(&tool_id) {
                    if let Some(joined_with) = joined_tools.get_mut(&previously_joined) {
                        joined_with.retain(|old_tool_id| old_tool_id != &tool_id);

                        if joined_with.is_empty() {
                            joined_tools.remove(&previously_joined);
                        }
                    }
                }

                // If this is a parent tool, then stop being a parent tool
                if let Some(joined_with) = joined_tools.remove(&tool_id) {
                    joined_with.into_iter()
                        .for_each(|old_tool| { join_parent_tool.remove(&old_tool); });
                }
            }

            SetDefaultForGroup(tool_group_id, default_tool_id) => {
                default_for_group.insert(tool_group_id, default_tool_id);
            }

            Select(tool_id) => {
                if let Some(_) = group_for_tool.get(&tool_id) {
                    // If the tool is joined to a set of other tools, we need to also select the tools in that group
                    let joined_tools    = joined_tools.get(&tool_id);
                    let tools_to_select = iter::once(tool_id).chain(joined_tools.into_iter().flatten().copied()).collect::<Vec<_>>();

                    // Deselect the tools in the list that aren't going to be immediately reselected
                    let tools_to_deselect = tools_to_select.iter()
                        .flat_map(|tool_id| group_for_tool.get(&tool_id).map(|group_id| (group_id, tool_id)))                               // Group for each tool
                        .flat_map(|(group_id, tool_id)| group_selection.get(group_id).map(|selected_tool_id| (selected_tool_id, tool_id)))  // Active selection for each group
                        .filter(|(selected_tool_id, new_tool_id)| selected_tool_id != new_tool_id)                                          // Remove tools that would just be immediately re-selected
                        .map(|(selected_tool_id, _)| selected_tool_id);                                                                     // Just the tools that need to be deselected

                    for deselect_tool_id in tools_to_deselect {
                        // Deselect the old tool in the state, location and owners
                        send_to_subscribers(Some(&mut subscribers), ToolState::Deselect(*deselect_tool_id)).await;
                        send_to_subscribers(tool_locations.get_mut(deselect_tool_id), ToolState::Deselect(*deselect_tool_id)).await;
                        send_to_subscribers(type_for_tool.get(deselect_tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::Deselect(*deselect_tool_id)).await;
                    }

                    // Select any tool that's in a group where the selection will change
                    let tools_to_select = tools_to_select.into_iter()
                        .flat_map(|tool_id| group_for_tool.get(&tool_id).map(|group_id| (*group_id, tool_id)))                              // Group for each tool
                        .filter(|(group_id, tool_id)| group_selection.get(group_id) != Some(tool_id))                                       // Filter to groups where the selected tool is changing
                        .collect::<Vec<_>>();

                    for (group_id, select_tool_id) in tools_to_select {
                        // Select the new tool
                        group_selection.insert(group_id, select_tool_id);

                        // Send to subscribers the new tool
                        send_to_subscribers(Some(&mut subscribers), ToolState::Select(select_tool_id)).await;
                        send_to_subscribers(tool_locations.get_mut(&select_tool_id), ToolState::Select(select_tool_id)).await;
                        send_to_subscribers(type_for_tool.get(&select_tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::Select(select_tool_id)).await;
                    }
                }
            }

            OpenDialog(tool_id) => {
                if tools.contains(&tool_id) {
                    if !open_dialogs.contains(&tool_id) {
                        // Mark the dialog as open
                        open_dialogs.insert(tool_id);

                        // Send to the owner that the dialog is open
                        send_to_subscribers(Some(&mut subscribers), ToolState::OpenDialog(tool_id)).await;
                        send_to_subscribers(tool_locations.get_mut(&tool_id), ToolState::OpenDialog(tool_id)).await;
                        send_to_subscribers(type_for_tool.get_mut(&tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::OpenDialog(tool_id)).await;
                    }
                }
            }

            CloseDialog(tool_id) => {
                if tools.contains(&tool_id) {
                    if open_dialogs.contains(&tool_id) {
                        // Mark the dialog as closed
                        open_dialogs.remove(&tool_id);

                        // Send to the owner that the dialog is open
                        send_to_subscribers(Some(&mut subscribers), ToolState::CloseDialog(tool_id)).await;
                        send_to_subscribers(tool_locations.get_mut(&tool_id), ToolState::CloseDialog(tool_id)).await;
                        send_to_subscribers(type_for_tool.get_mut(&tool_id).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::CloseDialog(tool_id)).await;
                    }
                }
            }

            CloseAllDialogs => {
                for closed_dialog in open_dialogs.drain() {
                    send_to_subscribers(Some(&mut subscribers), ToolState::CloseDialog(closed_dialog)).await;
                    send_to_subscribers(tool_locations.get_mut(&closed_dialog), ToolState::CloseDialog(closed_dialog)).await;
                    send_to_subscribers(type_for_tool.get_mut(&closed_dialog).and_then(|tool_type| tool_type_owners.get_mut(tool_type)), ToolState::CloseDialog(closed_dialog)).await;
                }
            }
        }
    }
}
