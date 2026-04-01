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

use flo_scene::*;

///
/// ID of the main document subprogram
///
pub fn subprogram_flowbetween_document() -> SubProgramId { SubProgramId::called("flowbetween::document::document") }

///
/// ID of the subprogram where DrawingWindowRequests can be sent to
///
pub fn subprogram_window() -> SubProgramId { SubProgramId::called("flowbetween::document::window") }

///
/// ID of the left tool dock program
///
pub fn subprogram_tool_dock_left() -> SubProgramId { SubProgramId::called("flowbetween::tool_dock::left") }

///
/// ID of the right tool dock program
///
pub fn subprogram_tool_dock_right() -> SubProgramId { SubProgramId::called("flowbetween::tool_dock::right") }

///
/// ID of the 'floating' tools program
///
pub fn subprogram_floating_tools() -> SubProgramId { SubProgramId::called("flowbetween::tools::floating") }
