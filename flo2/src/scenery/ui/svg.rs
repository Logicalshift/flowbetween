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
use flo_svg::*;

///
/// Returns the rendering instructions for an SVG file, with the center of the view box at 0,0
///
pub fn svg(svg: &[u8]) -> Vec<Draw> {
    // Convert to a string
    let svg         = str::from_utf8(svg).unwrap();

    // Generate the main drawing instructions
    let mut drawing = vec![];
    let document    = parse_svg(svg, &mut drawing).unwrap();

    if let Some(((min_x, min_y), (max_x, max_y))) = document.viewbox() {
        // Translate the center of the viewbox to the 0,0 position
        let center_pos  = ((min_x+max_x)/2.0, ((min_y+max_y)/2.0));
        let translation = Transform2D::translate(-center_pos.0, -center_pos.1);

        drawing.splice(0..0, vec![
            Draw::PushState,
            Draw::MultiplyTransform(translation),
        ]);

        drawing.push(Draw::PopState);
    }

    drawing
}

///
/// Returns the rendering instructions for an SVG file, with the center of the view box at 0,0 and scaled to the specified width
///
pub fn svg_with_width(svg: &[u8], width: f64) -> Vec<Draw> {
    // Convert to a string
    let svg         = str::from_utf8(svg).unwrap();

    // Generate the main drawing instructions
    let mut drawing = vec![];
    let document    = parse_svg(svg, &mut drawing).unwrap();

    if let Some(((min_x, min_y), (max_x, max_y))) = document.viewbox() {
        // Translate the center of the viewbox to the 0,0 position
        let center_pos  = ((min_x+max_x)/2.0, ((min_y+max_y)/2.0));
        let translation = Transform2D::translate(-center_pos.0, -center_pos.1);
        let scale       = (width as f32)/(max_x-min_x);
        let scale       = Transform2D::scale(scale, scale);

        drawing.splice(0..0, vec![
            Draw::PushState,
            Draw::MultiplyTransform(scale * translation),
        ]);

        drawing.push(Draw::PopState);
    }

    drawing
}
