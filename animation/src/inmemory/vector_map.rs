use super::super::traits::*;

use std::sync::*;
use std::collections::HashMap;

///
/// Maps element IDs to vectors. Clones share the map.
/// 
#[derive(Clone)]
pub struct VectorMap {
    // Maps element IDs to vectors
    map: Arc<Mutex<HashMap<ElementId, Vector>>>
}

impl VectorMap {
    ///
    /// Creates a new vector map
    /// 
    pub fn new() -> VectorMap {
        VectorMap {
            map: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    ///
    /// Sets the vector with the specified element ID
    /// 
    pub fn set_vector_for_id(&self, element_id: ElementId, vector: Vector) {
        self.map.lock().unwrap().insert(element_id, vector);
    }
    
    ///
    /// Retrieves the vector with the specified element ID
    /// 
    pub fn vector_with_id(&self, element_id: ElementId) -> Option<Vector> {
        let map = self.map.lock().unwrap();

        map.get(&element_id).cloned()
    }
}
