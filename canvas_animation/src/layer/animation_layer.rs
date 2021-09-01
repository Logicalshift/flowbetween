use crate::path::*;
use super::cache::*;

use ::desync::*;
use flo_canvas::*;

use std::mem;
use std::sync::*;
use std::time::{Duration};

///
/// Represents an animated layer of a vector drawing. This accepts commands in the form
/// of `Draw` instructions, although it will only render to a single layer in the finished
/// rendering: sprite and layer commands will be ignored.
///
pub struct AnimationLayer {
    /// The current state of the layer drawing
    layer_state: LayerDrawingToPaths,

    /// The drawing that has been performed so far
    drawing: Vec<AnimationPath>,

    /// The cached drawing, if it exists
    cached_paths: Mutex<Option<Arc<Vec<AnimationPath>>>>,

    /// The state cache for this layer
    cache: Desync<AnimationLayerCache>
}

impl AnimationLayer {
    ///
    /// Creates an empty animation layer
    ///
    pub fn new() -> AnimationLayer {
        AnimationLayer {
            layer_state:    LayerDrawingToPaths::new(),
            drawing:        vec![],
            cached_paths:   Mutex::new(None),
            cache:          Desync::new(AnimationLayerCache::new())
        }
    }

    ///
    /// Clears this layer
    ///
    pub fn clear(&mut self) {
        self.drawing.clear();
        self.drawing.extend(self.layer_state.draw([Draw::ClearLayer]));
    }

    ///
    /// Sets the time that paths added to this layer should appear
    ///
    pub fn set_time(&mut self, drawing_time: Duration) {
        self.layer_state.set_time(drawing_time);
    }

    ///
    /// Adds a new path to this layer
    ///
    pub fn add_path(&mut self, path: AnimationPath) {
        self.drawing.push(path);
    }

    ///
    /// Retrieves a pointer to the drawing for this layer
    ///
    fn get_cached_paths(&mut self) -> Arc<Vec<AnimationPath>> {
        if let Some(cached_paths) = &(*self.cached_paths.lock().unwrap()) {
            // We've already got the drawing instructions in a cached reference
            Arc::clone(cached_paths)
        } else {
            // Move the drawing instructions to a reference
            let cached_paths                    = mem::take(&mut self.drawing);
            let cached_paths                    = Arc::new(cached_paths);
            *self.cached_paths.lock().unwrap()  = Some(Arc::clone(&cached_paths));

            // Return the newly cached drawing
            cached_paths
        }
    }

    ///
    /// Adds drawing onto this layer
    ///
    pub fn draw<DrawIter: IntoIterator<Item=Draw>>(&mut self, drawing: DrawIter) {
        // Release the drawing instructions from the cache if necessary
        if let Some(mut cached_paths) = (*self.cached_paths.lock().unwrap()).take() {
            if let Some(cached_paths) = Arc::get_mut(&mut cached_paths) {
                // Swap out the drawing with the cached version
                self.drawing.clear();
                mem::swap(&mut self.drawing, cached_paths);
            } else {
                // Clone out the drawing
                self.drawing = (*cached_paths).clone();
            }

            // Clear the cache whenever we remove the cached paths
            self.cache.desync(|cache| cache.flush());
        }

        // Render to the drawing
        self.drawing.extend(self.layer_state.draw(drawing));
    }

    ///
    /// Starts filling the cache in the background
    ///
    pub fn fill_cache(&mut self) {
    }
}

impl Clone for AnimationLayer {
    fn clone(&self) -> Self {
        let cached_paths = self.cached_paths.lock().unwrap().clone();

        AnimationLayer {
            layer_state:    self.layer_state.clone(),
            drawing:        self.drawing.clone(),
            cached_paths:   Mutex::new(cached_paths),
            cache:          Desync::new(AnimationLayerCache::new())
        }
    }
}
