use crate::scenery::ui::*;

use uuid::{uuid};

///
/// The canvas tool group contains the tools that directly affect the canvas (eg: the brush tool, selection tool, etc)
///
pub const TOOL_GROUP_CANVAS:        ToolGroupId = ToolGroupId::with_id(uuid!("55B369A1-F118-4F91-90DC-890DFF24A9A2"));

///
/// The colour group is the tool that determines the fill colour of the current brush
///
pub const TOOL_GROUP_COLOUR:        ToolGroupId = ToolGroupId::with_id(uuid!("10B9C491-4977-4AAA-BEE6-00FABA2212A6"));

///
/// The brush size group determines the overall size of the current brush
///
pub const TOOL_GROUP_BRUSHSIZE:     ToolGroupId = ToolGroupId::with_id(uuid!("E31328AC-F5F7-492B-A6F3-525CF02405D0"));

///
/// The layer group determines which layer the user is drawing on
///
pub const TOOL_GROUP_LAYER:         ToolGroupId = ToolGroupId::with_id(uuid!("7A8DB3E7-AF1A-4ECC-B69B-1D48FE1EE438"));

///
/// The effects group applies other effects to the brush stroke
///
pub const TOOL_GROUP_EFFECTS:       ToolGroupId = ToolGroupId::with_id(uuid!("20A70C17-048D-43F6-A5DB-19DC39EDA737"));

///
/// The outline group describes the line to draw around what's drawn by the user
///
pub const TOOL_GROUP_OUTLINE:       ToolGroupId = ToolGroupId::with_id(uuid!("A9A3763F-5D78-4C8E-8676-A334803A1E57"));

///
/// The reference tool is used to determine which reference layers are displayed to the user
///
pub const TOOL_GROUP_REFERENCE:     ToolGroupId = ToolGroupId::with_id(uuid!("D6BDA987-7C9A-4018-ADFE-9E4C66837359"));

///
/// The annotation tool is used to determine which 'non-photo' layers are displayed to the user
///
pub const TOOL_GROUP_ANNOTATIONS:   ToolGroupId = ToolGroupId::with_id(uuid!("8098497D-384C-4E78-8A72-A9456BA0A2DF"));

///
/// The onion skin tool determines how previous/next frames are displayed to the user
///
pub const TOOL_GROUP_ONIONSKIN:     ToolGroupId = ToolGroupId::with_id(uuid!("EA69ECE1-AA11-4A82-80BB-DB1554C77EAA"));
