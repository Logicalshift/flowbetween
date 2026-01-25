use ::serde::*;
use uuid::*;

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
}
