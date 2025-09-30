//!
//! The tool_state subprogram manages the overall state of where tools are located in a document. The state
//! can be monitored or queried in order to draw the tools, or updated to set which tools are available
//! in the application
//!

use flo_scene::*;
use flo_draw::canvas::*;

use ::serde::*;
use uuid::*;

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
