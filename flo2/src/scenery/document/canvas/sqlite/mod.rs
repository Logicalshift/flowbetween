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

mod canvas;
mod canvas_brushes;
mod canvas_layers;
mod canvas_properties;
mod canvas_queries;
mod canvas_shapes;
mod canvas_program;
mod error;
mod id_cache;

pub use canvas::*;
pub use canvas_program::*;

#[cfg(test)]
mod test_canvas;

#[cfg(test)]
mod test_canvas_program;
