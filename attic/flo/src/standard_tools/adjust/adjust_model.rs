use crate::tools::*;

use std::sync::*;

///
/// The model for the Adjust tool
///
pub struct AdjustModel {
    /// The future runs the adjust tool
    pub (super) future: Mutex<ToolFuture>,
}
