use super::vector_layer::*;
use super::super::traits::*;

use std::collections::HashMap;

///
/// The core in-memory animation data structure
/// 
pub struct AnimationCore {
    /// The edit log for this animation
    pub edit_log: Vec<AnimationEdit>,

    /// The next element ID to assign
    pub next_element_id: i64,

    /// The size of the animation canvas
    pub size: (f64, f64),

    /// The vector layers in this animation
    pub vector_layers: HashMap<u64, InMemoryVectorLayer>,
}
