// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::scenery::ui::*;

use uuid::{uuid};

pub const TOOL_BRUSH:           ToolTypeId = ToolTypeId::with_id(uuid!("1A318AE4-3CF9-4056-B3CC-FF94C7899C2C"));
pub const TOOL_ERASER:          ToolTypeId = ToolTypeId::with_id(uuid!("EB0FB0B5-6C2F-4D25-A712-F678F3002FBE"));
pub const TOOL_NONPHOTO_PENCIL: ToolTypeId = ToolTypeId::with_id(uuid!("C50FE867-7AF7-48FD-B8B5-5890C4281EA8"));
pub const TOOL_PAINT_BUCKET:    ToolTypeId = ToolTypeId::with_id(uuid!("D733FABA-F47B-44F9-B2AC-D67A9CCF5ECF"));
pub const TOOL_LASSO:           ToolTypeId = ToolTypeId::with_id(uuid!("095BB2DF-995A-40EF-B724-05915AEDD230"));
pub const TOOL_ELLIPSE:         ToolTypeId = ToolTypeId::with_id(uuid!("EC624811-4DDF-410C-86C8-83B8ECA87A0E"));
pub const TOOL_RECTANGLE:       ToolTypeId = ToolTypeId::with_id(uuid!("74E31580-0416-4BF2-B406-77E397652EE3"));
pub const TOOL_POLYGON:         ToolTypeId = ToolTypeId::with_id(uuid!("E37017DE-D6A7-447A-8898-C38322198182"));
