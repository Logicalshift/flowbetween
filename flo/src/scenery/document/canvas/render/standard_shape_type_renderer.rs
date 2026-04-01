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

use super::shape_type_renderer::*;
use super::super::basic_properties::*;
use super::super::property::*;

use flo_draw::canvas::*;
use flo_scene::*;

///
/// Runs the standard shape renderer program (for `ShapeType::default().render_program_id()`)
///
/// This renders shapes using the properties defined in basic_properties. Groups are just rendered in a 'straight through' fashion
///
pub async fn standard_shape_type_renderer_program(input: InputStream<RenderShapesRequest>, context: SceneContext) {
    shape_renderer_program(input, context, |shape, _frame_time, drawing| {
        // Generate the path
        drawing.new_path();
        shape.shape.to_path()
            .into_iter()
            .for_each(|path| drawing.bezier_path(&path));

        // Set up the properties to render it
        let fill    = FlatFill::from_properties(shape.properties.iter());
        let stroke  = Stroke::from_properties(shape.properties.iter());

        if let Some(fill) = fill {
            fill.draw(drawing);
        }

        if let Some(stroke) = stroke {
            stroke.draw(drawing);
        }

        // Any grouped shapes are rendered after
        shape.group.iter()
            .for_each(|(_shape, shape_drawing)| drawing.extend(shape_drawing.iter().cloned()));
    }).await;
}
