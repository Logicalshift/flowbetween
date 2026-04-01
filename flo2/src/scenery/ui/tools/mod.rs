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
//! Tools in FlowBetween provide different ways of interacting with the canvas. They include obvious things
//! like, say, a brush or a selection tool, but FlowBetween generalises the concept so that things like the
//! colour and layer selection are also tools.
//!
//! Every tool has a group. Only one tool can be selected within a group, so groups might represent things
//! like the main tool, the colour, line properties, layer selections, etc. The idea here is that the
//! basic operation is very similar to how other 'canvas' type apps works where you pick a tool and its
//! properties, but tools from different groups can be joined together so the user can switch multiple
//! properties all at once.
//!
//! Tools can 'live' in multiple places. They typically start out in a tool dock, which is just a fixed
//! region on the left or right of the document that shows the icons for selected tools.
//!

pub (crate) mod tool_state;
pub (crate) mod tool_dock;
pub (crate) mod floating_tool_dock;
pub (crate) mod tool_graphics;
pub (crate) mod sprite_manager;

pub (crate) mod physics_simulation;
pub (crate) mod physics_simulation_joints;

pub (crate) mod physics_simulation_object;

pub use tool_state::*;

#[cfg(test)] mod test;
