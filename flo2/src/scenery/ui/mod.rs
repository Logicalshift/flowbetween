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

//!
//! # UI
//!
//! This defines subprograms that can run parts of the UI for FlowBetween. These subprograms are likely re-usable in other projects.
//!

mod namespaces;
mod control;
mod control_id;
mod focus;
mod focus_events;
mod subprograms;
mod ui_path;
mod dialog_id;
mod dialog;
mod egui;
mod tools;
mod colors;
mod render_binding;
mod svg;

pub use colors::*;
pub use namespaces::*;
pub use control::*;
pub use control_id::*;
pub use focus::*;
pub use focus_events::*;
pub use subprograms::*;
pub use ui_path::*;
pub use dialog_id::*;
pub use dialog::*;
pub use egui::*;
pub use tools::tool_state::*;
pub use tools::physics_simulation::*;
pub use tools::physics_simulation_object::*;
pub use tools::physics_simulation_joints::*;
pub use render_binding::*;
pub use svg::*;

pub use tools::tool_dock::*;
pub use tools::floating_tool_dock::*;
