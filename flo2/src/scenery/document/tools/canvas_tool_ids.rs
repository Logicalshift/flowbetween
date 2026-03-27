use crate::scenery::ui::*;

use uuid::{uuid};

pub const TOOL_BRUSH:           ToolId = ToolId::with_id(uuid!("1A318AE4-3CF9-4056-B3CC-FF94C7899C2C"));
pub const TOOL_ERASER:          ToolId = ToolId::with_id(uuid!("EB0FB0B5-6C2F-4D25-A712-F678F3002FBE"));
pub const TOOL_NONPHOTO_PENCIL: ToolId = ToolId::with_id(uuid!("C50FE867-7AF7-48FD-B8B5-5890C4281EA8"));
pub const TOOL_PAINT_BUCKET:    ToolId = ToolId::with_id(uuid!("D733FABA-F47B-44F9-B2AC-D67A9CCF5ECF"));
pub const TOOL_LASSO:           ToolId = ToolId::with_id(uuid!("095BB2DF-995A-40EF-B724-05915AEDD230"));
pub const TOOL_ELLIPSE:         ToolId = ToolId::with_id(uuid!("EC624811-4DDF-410C-86C8-83B8ECA87A0E"));
pub const TOOL_RECTANGLE:       ToolId = ToolId::with_id(uuid!("74E31580-0416-4BF2-B406-77E397652EE3"));
pub const TOOL_POLYGON:         ToolId = ToolId::with_id(uuid!("E37017DE-D6A7-447A-8898-C38322198182"));
