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
