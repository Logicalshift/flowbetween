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

use super::frame_time::*;
use super::property::*;
use super::shape::*;

use ::serde::*;
use uuid::*;

use std::str::*;
use std::fmt;

///
/// Identifier used for a layer in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasLayerId(Uuid);

impl CanvasLayerId {
    ///
    /// Creates a unique new canvas layer ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    ///
    /// Creates a layer ID from a string
    ///
    pub fn from_string(string_guid: &str) -> Self {
        Self(Uuid::from_str(string_guid).unwrap())
    }

    ///
    /// Returns the string representation of this layer ID
    ///
    #[inline]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for CanvasLayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<CanvasShapeParent> for (CanvasLayerId, FrameTime) {
    fn into(self) -> CanvasShapeParent {
        CanvasShapeParent::Layer(self.0, self.1)
    }
}

impl Into<CanvasPropertyTarget> for CanvasLayerId {
    fn into(self) -> CanvasPropertyTarget {
        CanvasPropertyTarget::Layer(self)
    }
}
