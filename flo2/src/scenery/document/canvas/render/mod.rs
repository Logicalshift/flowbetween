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

mod canvas_render_program;
mod layer_renderer;
mod shape_type_renderer;
mod shape_renderer;
mod standard_shape_type_renderer;

pub use canvas_render_program::*;
pub use layer_renderer::*;
pub use shape_renderer::*;
pub use shape_type_renderer::*;
pub use standard_shape_type_renderer::*;

#[cfg(test)]
mod test_shape_renderer;

#[cfg(test)]
mod test_layer_renderer;

#[cfg(test)]
mod test_canvas_render_program;
