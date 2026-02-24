use super::property::*;

use ::serde::*;
use uuid::*;

use std::str::*;
use std::fmt;

///
/// Identifier used for a brush in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasBrushId(Uuid);

impl CanvasBrushId {
    ///
    /// Creates a unique new canvas brush ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    ///
    /// Creates a brush ID from a string
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

impl fmt::Display for CanvasBrushId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<CanvasPropertyTarget> for CanvasBrushId {
    fn into(self) -> CanvasPropertyTarget {
        CanvasPropertyTarget::Brush(self)
    }
}
