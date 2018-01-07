use super::super::vector::*;

use std::time::Duration;

///
/// Represents a layer that contains vector elements
/// 
pub trait VectorLayer : Send {
    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, when: Duration, new_element: Box<VectorElement>);
}
