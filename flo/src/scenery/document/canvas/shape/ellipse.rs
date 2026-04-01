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
/// Serialized form of an ellipse in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasEllipseV1 {
    pub min:        CanvasPoint,
    pub max:        CanvasPoint,
    pub direction:  CanvasPoint,
}

pub type CanvasEllipse = CanvasEllipseV1;

/// Shape type to indicate a shape encoded in V1 canvas ellipse format
pub const CANVAS_ELLIPSE_V1_TYPE: i64 = 2;
