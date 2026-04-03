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
/// ID of the main 'focus' subprogram
///
pub fn subprogram_dialog() -> SubProgramId { SubProgramId::called("flowbetween::ui::dialog") }

///
/// ID of the tool state subprogram
///
pub fn subprogram_tool_state() -> SubProgramId { SubProgramId::called("flowbetween::tool_state") }

///
/// ID of the physics layer subprogram
///
pub fn subprogram_physics_layer() -> SubProgramId { SubProgramId::called("flowbetween::ui::physics_layer") }
