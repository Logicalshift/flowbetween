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

use super::brush_response::*;

use crate::scenery::document::canvas::*;

use flo_scene::*;
use flo_scene::programs::*;

use serde::*;

///
/// A request to run a brush command
///
/// Brushes work as a pipeline of commands, converting from raw brush inputs to canvas shapes
///
#[derive(Serialize, Deserialize)]
pub enum BrushRequest {
    /// Requests to start a new brush stroke. The supplied stream target recieves the brush responses.
    ///
    /// Brush responses don't directly provide the shapes for the brush but rather a pipeline that can be used to process
    /// the raw brush points into a brush stroke.
    RunBrush(CanvasBrushId, StreamTarget),
}

impl SceneMessage for BrushRequest {
}

impl QueryRequest for BrushRequest {
    type ResponseData = BrushResponse;

    fn with_new_target(self, new_target: StreamTarget) -> Self {
        match self {
            Self::RunBrush(brush_id, _) => Self::RunBrush(brush_id, new_target)
        }
    }
}
