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

use super::brush_point::*;

use futures::prelude::*;

///
/// Converts an input stream of brush points to an output stream of brush points with a radius based on the pen pressure
///
pub fn brush_pressure_to_radius_linear(max_radius: f64, input_stream: impl 'static + Send + Stream<Item=BrushPoint>) -> impl 'static + Send + Stream<Item=BrushPoint> {
    input_stream.map(move |point| {
        let mut point = point;

        point.daub_radius = Some((point.pressure.max(0.0).min(1.0)) * max_radius);

        point
    })
}

