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
    frame_duration: Duration
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
            frame_duration: Duration::from_millis(1000/30)
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
    fn open(&self) -> Option<Editor<AnimationLayers+'static>> { None }
    fn read(&self) -> Option<Reader<AnimationLayers+'static>> { None }
}

impl AnimationSize for AnimationCore {
    fn size(&self) -> (f64, f64) { self.size }

    fn set_size(&mut self, new_size: (f64, f64)) {
        self.size = new_size;
    }
}
