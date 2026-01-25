use ::serde::*;
use uuid::*;

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
}
