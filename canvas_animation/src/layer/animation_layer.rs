use super::cache::*;

use crate::path::*;
use crate::region::*;

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

    /// The regions in this layer
    regions: Vec<Arc<dyn AnimationRegion>>,

    /// The cached paths, if they exist (as shared amongst pending caching operations)
    cached_paths: Option<Arc<Vec<AnimationPath>>>,

    /// The cached regions, if they exist (as shared amongst pending caching operations)
    cached_regions: Option<Arc<Vec<Arc<dyn AnimationRegion>>>>,

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
            regions:        vec![],
            cached_paths:   None,
            cached_regions: None,
            cache:          Desync::new(AnimationLayerCache::new())
        }
    }

    ///
    /// Clears this layer of all animation regions
    ///
    pub fn clear_regions(&mut self) {
        self.cached_regions = None;
        self.regions.clear();

        self.cache.desync(|cache| cache.flush());
    }

    ///
    /// Adds an animation region to this layer
    ///
    pub fn add_region<Region: 'static+AnimationRegion>(&mut self, region: Region) {
        // Release the regions from the cache if necessary
        if let Some(mut cached_regions) = self.cached_regions.take() {
            if let Some(cached_regions) = Arc::get_mut(&mut cached_regions) {
                // Swap out the regions with the cached version
                self.regions.clear();
                mem::swap(&mut self.regions, cached_regions);
            } else {
                // Clone out the drawing
                self.regions = (*cached_regions).clone();
            }

            // Clear the cache whenever we remove the cached paths
            self.cache.desync(|cache| cache.flush());
        }

        // Add to the list of regions in this layer
        self.regions.push(Arc::new(region));
    }

    ///
    /// Retrieves a pointer to the drawing for this layer
    ///
    fn get_cached_regions(&mut self) -> Arc<Vec<Arc<dyn AnimationRegion>>> {
        if let Some(cached_regions) = &self.cached_regions {
            // We've already got the paths in a cached reference
            Arc::clone(cached_regions)
        } else {
            // Move the paths to a reference
            let cached_regions  = mem::take(&mut self.regions);
            let cached_regions  = Arc::new(cached_regions);
            self.cached_regions = Some(Arc::clone(&cached_regions));

            // Return the newly cached paths
            cached_regions
        }
    }

    ///
    /// Clears this layer of all drawing operations
    ///
    pub fn clear_drawing(&mut self) {
        self.cached_paths = None;
        self.drawing.clear();
        self.drawing.extend(self.layer_state.draw([Draw::ClearLayer]));

        self.cache.desync(|cache| cache.flush());
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
        if let Some(cached_paths) = &self.cached_paths {
            // We've already got the paths in a cached reference
            Arc::clone(cached_paths)
        } else {
            // Move the paths to a reference
            let cached_paths    = mem::take(&mut self.drawing);
            let cached_paths    = Arc::new(cached_paths);
            self.cached_paths   = Some(Arc::clone(&cached_paths));

            // Return the newly cached paths
            cached_paths
        }
    }

    ///
    /// Adds drawing onto this layer
    ///
    pub fn draw<DrawIter: IntoIterator<Item=Draw>>(&mut self, drawing: DrawIter) {
        // Release the paths from the cache if necessary
        if let Some(mut cached_paths) = self.cached_paths.take() {
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
    /// Starts filling the cache in the background, in preparation for future operations
    ///
    pub fn fill_cache(&mut self) {
        let cached_paths    = self.get_cached_paths();
        let cached_regions  = self.get_cached_regions();

        self.cache.desync(move |cache| {
            if cache.drawing_bounding_boxes.is_none()   { cache.calculate_bounding_boxes(&*cached_paths); }
            if cache.drawing_times.is_none()            { cache.calculate_drawing_times(&*cached_paths); }
            if cache.region_bounding_boxes.is_none()    { cache.calculate_region_bounding_boxes(&*cached_paths, &*cached_regions); }
            if cache.paths_for_region.is_none()         { cache.cut_drawing_into_regions(&*cached_paths, &*cached_regions); }
        });
    }
}

impl Clone for AnimationLayer {
    fn clone(&self) -> Self {
        AnimationLayer {
            layer_state:    self.layer_state.clone(),
            drawing:        self.drawing.clone(),
            regions:        self.regions.clone(),
            cached_paths:   self.cached_paths.clone(),
            cached_regions: self.cached_regions.clone(),
            cache:          Desync::new(AnimationLayerCache::new())
        }
    }
}
