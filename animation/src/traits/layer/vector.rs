use super::super::vector::*;
use super::super::brush::*;

use std::time::Duration;
use std::sync::*;

///
/// Represents a layer that contains vector elements
/// 
pub trait VectorLayer : Send {
    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, when: Duration, new_element: Box<VectorElement>);

    ///
    /// The brush that will be active for the next element that's added to this layer
    /// 
    fn active_brush(&self, when: Duration) -> Arc<Brush>;
}
