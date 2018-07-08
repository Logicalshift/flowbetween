use super::vector_map::*;
use super::super::traits::*;

use std::time::Duration;
use std::sync::*;

///
/// A keyframe in a vector layer
/// 
pub struct VectorKeyFrame {
    core: Mutex<VectorKeyFrameCore>
}

///
/// Data storage for a vector keyframe
/// 
struct VectorKeyFrameCore {
    /// When this frame starts
    start_time: Duration,

    /// The elements in this key frame (ordered from back to front)
    elements: Vec<(Duration, ElementId)>,

    /// Maps IDs to the corresponding vector
    vector_map: VectorMap,

    /// The properties that will apply to the next element added to this core
    active_properties: Arc<VectorProperties>
}

impl VectorKeyFrame {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration, vector_map: VectorMap) -> VectorKeyFrame {
        VectorKeyFrame {
            core: Mutex::new(VectorKeyFrameCore::new(start_time, vector_map))
        }
    }

    ///
    /// The start time of this key frame
    /// 
    pub fn start_time(&self) -> Duration {
        self.core.lock().unwrap().start_time()
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    pub fn add_element(&self, when: Duration, new_element: Vector) {
        self.core.lock().unwrap().add_element(when, new_element);
    }

    ///
    /// Retrieves the elements in this keyframe
    /// 
    pub fn elements(&self) -> Vec<(Duration, Vector)> {
        let core            = self.core.lock().unwrap();

        let elements        = core.elements.iter()
            .filter_map(|(duration, element_id)| core.vector_map.vector_with_id(*element_id).map(|vector| (*duration, vector)));

        elements.collect()
    }

    ///
    /// Finds an element in this frame and when it appears
    /// 
    pub fn element_with_id(&self, element_id: ElementId) -> Option<(Duration, Vector)> {
        let core = self.core.lock().unwrap();

        let element = core.vector_map
            .vector_with_id(element_id)
            .and_then(|vector| {
                let appearance_time = core.elements.iter().filter(|(_, id)| id == &element_id).nth(0);

                appearance_time.map(|(appearance_time, _)| (*appearance_time, vector))
            });
        
        element
    }

    ///
    /// Retrieves the properties that will be applied to the next element added to this keyframe
    /// 
    #[inline]
    pub fn active_properties(&self) -> VectorProperties {
        self.core.lock().unwrap().active_properties().clone()
    }
}

impl VectorKeyFrameCore {
    ///
    /// Creates a new vector key frame
    /// 
    pub fn new(start_time: Duration, vector_map: VectorMap) -> VectorKeyFrameCore {
        VectorKeyFrameCore {
            start_time:         start_time,
            elements:           vec![],
            vector_map:         vector_map,
            active_properties:  Arc::new(VectorProperties::default())
        }
    }

    ///
    /// The start time of this key frame
    /// 
    #[inline]
    pub fn start_time(&self) -> Duration {
        self.start_time
    }

    ///
    /// Retrieves the properties that will be applied to the next element added to this keyframe
    /// 
    #[inline]
    pub fn active_properties<'a>(&'a self) -> &'a VectorProperties {
        &self.active_properties
    }

    ///
    /// Adds a new element to the front of the vector
    /// 
    #[inline]
    pub fn add_element(&mut self, when: Duration, new_element: Vector) {
        let element_id = new_element.id();

        self.active_properties = new_element.update_properties(Arc::clone(&self.active_properties));
        self.vector_map.set_vector_for_id(element_id, new_element);
        self.elements.push((when, element_id));
    }
}
