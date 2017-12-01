use super::vector_layer::*;
use super::super::traits::*;

use std::sync::*;
use std::time::Duration;

///
/// Core values associated with an animation
/// 
struct AnimationCore {
    /// The size of the animation canvas
    size: (f64, f64),

    /// The duration of a frame in the animation
    frame_duration: Duration,

    /// The layers in this animation
    layers: Vec<Arc<Layer>>,

    next_layer_id: u64
}

///
/// Represents an animation that's stored entirely in memory 
///
pub struct InMemoryAnimation {
    /// The core contains the actual animation data
    core: RwLock<AnimationCore>
}

impl InMemoryAnimation {
    pub fn new() -> InMemoryAnimation {
        // Create the core (30fps by default)
        let core = AnimationCore { 
            size:           (1980.0, 1080.0),
            frame_duration: Duration::from_millis(1000/30),
            layers:         vec![],
            next_layer_id:  0
        };

        // Create the final animation
        InMemoryAnimation { core: RwLock::new(core) }
    }
}

impl Animation for InMemoryAnimation { }

impl Editable<AnimationSize+'static> for InMemoryAnimation {
    fn open(&self) -> Option<Editor<AnimationSize+'static>> {
        // (Need the explicit typing here as rust can't figure it out implicitly)
        let core: &RwLock<AnimationSize>    = &self.core;
        let core                            = core.write().unwrap();

        Some(Editor::new(core))
    }

    fn read(&self) -> Option<Reader<AnimationSize+'static>> {
        let core: &RwLock<AnimationSize>    = &self.core;
        let core                            = core.read().unwrap();

        Some(Reader::new(core))
    }
}

impl Editable<AnimationLayers+'static> for InMemoryAnimation {
    fn open(&self) -> Option<Editor<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &self.core;

        Some(Editor::new(core.write().unwrap()))
    }

    fn read(&self) -> Option<Reader<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &self.core;

        Some(Reader::new(core.read().unwrap()))
    }
}

impl AnimationSize for AnimationCore {
    fn size(&self) -> (f64, f64) { self.size }

    fn set_size(&mut self, new_size: (f64, f64)) {
        self.size = new_size;
    }
}

impl AnimationLayers for AnimationCore {
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Layer>> {
        Box::new(self.layers.iter().map(|x| &**x))
    }

    fn remove_layer(&mut self, layer_id: u64) {
        // Find the index of the layer with this ID
        let remove_index = {
            let mut remove_index = None;

            for index in 0..self.layers.len() {
                if self.layers[index].id() == layer_id {
                    remove_index = Some(index);
                }
            }
            remove_index
        };

        // Remove this layer
        if let Some(remove_index) = remove_index {
            self.layers.remove(remove_index);
        }
    }

    fn add_new_layer<'a>(&'a mut self) -> &'a Layer {
        // Pick an ID for this layer
        let layer_id = self.next_layer_id;
        self.next_layer_id += 1;

        // Generate the layer
        let new_layer = Arc::new(VectorLayer::new(layer_id));
        self.layers.push(new_layer);

        // Result is a reference to the layer
        &**self.layers.last().unwrap()
    }
}