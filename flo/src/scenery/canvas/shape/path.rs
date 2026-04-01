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

use super::super::point::*;

use ::serde::*;

///
/// Serialized form of a bezier path in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPathV1 {
    pub start_point: CanvasPoint,
    pub actions:     Vec<CanvasPathV1Action>,
}

///
/// Actions for each point on a v1 path (except the first point, which is always a 'move' action)
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CanvasPathV1Action {
    /// Draws a line to the start point of the current subpath
    Close,

    /// Starts a subpath
    Move(CanvasPoint),

    /// Draws a line to the specified point
    Line(CanvasPoint),

    /// Creates a quadratic bezier curve to the specified point
    QuadraticCurve { end: CanvasPoint, cp: CanvasPoint },

    /// Creates a cubic bezier curve to the specified point
    CubicCurve { end: CanvasPoint, cp1: CanvasPoint, cp2: CanvasPoint },
}

pub type CanvasPath = CanvasPathV1;

/// Shape type to indicate a shape encoded in V1 canvas path format
pub const CANVAS_PATH_V1_TYPE: i64 = 0;
