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

use flo_draw::canvas::*;

#[inline] pub fn color_tool_border() -> Color                   { Color::Rgba(0.6, 0.6, 0.6, 1.0) }
#[inline] pub fn color_tool_shadow() -> Color                   { Color::Rgba(0.4, 0.4, 0.4, 0.4) }
#[inline] pub fn color_tool_outline() -> Color                  { Color::Rgba(0.4, 0.7, 1.0, 1.0) }
#[inline] pub fn color_tool_background() -> Color               { Color::Rgba(0.7, 0.7, 0.7, 0.9) }
#[inline] pub fn color_tool_background_selected() -> Color      { Color::Rgba(0.5, 0.6, 0.6, 0.9) }
#[inline] pub fn color_tool_background_highlighted() -> Color   { Color::Rgba(0.8, 0.85, 0.85, 0.9) }
#[inline] pub fn color_tool_border_selected() -> Color          { Color::Rgba(0.6, 0.7, 0.8, 1.0) }

#[inline] pub fn color_tool_dock_background() -> Color          { Color::Rgba(0.2, 0.2, 0.2, 0.95) }
#[inline] pub fn color_tool_dock_outline() -> Color             { Color::Rgba(0.8, 0.8, 0.8, 1.0) }
#[inline] pub fn color_tool_dock_highlight() -> Color           { Color::Rgba(0.1, 0.1, 0.1, 1.0) }
#[inline] pub fn color_tool_dock_selected() -> Color            { Color::Rgba(0.0, 0.0, 0.0, 1.0) }

#[inline] pub fn color_brush_preview() -> Color                 { Color::Rgba(0.8, 0.8, 0.8, 0.95) }
