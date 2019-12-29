use super::super::brush::*;

use std::time::Duration;
use std::sync::*;

///
/// Represents a layer that contains vector elements
///
pub trait VectorLayer : Send {
    ///
    /// The brush that will be active for the next element that's added to this layer (if one is set)
    ///
    fn active_brush(&self, when: Duration) -> Option<Arc<dyn Brush>>;
}
