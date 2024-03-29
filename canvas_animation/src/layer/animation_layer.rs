use super::cache::*;

use crate::path::*;
use crate::region::*;

use ::desync::*;
use flo_canvas::*;

use futures::prelude::*;

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

///
/// Graphics context for rendering to an animation layer
///
pub struct AnimationLayerContext<'a> {
    /// The layer that's being rendered to
    animation_layer: &'a mut AnimationLayer,

    /// Cached drawing instructions (dumped to the layer periodically)
    cache: Vec<Draw>
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

    ///
    /// Generates the rendering instructions for this layer at a particular time
    ///
    pub fn render_at_time<'a>(&'a mut self, time: Duration) -> impl 'a+Future<Output=Vec<Draw>> {
        // Ensure that all of the cached values are available
        self.fill_cache();

        // Fetch the regions and the paths, ready for rendering
        let regions = self.get_cached_regions();

        // Process the regions to generate the final rendering
        async move {
            self.cache.future_sync(move |cache| {
                async move {
                    let mut rendering = vec![];
                    cache.render_at_time(time, &*regions, &mut rendering);

                    rendering
                }.boxed()
            })
            .await
            .unwrap()
        }
    }

    ///
    /// Renders this layer synchronously to a graphics context
    ///
    pub fn render_sync<Context: Send+GraphicsContext+?Sized>(&mut self, time: Duration, gc: &mut Context) {
        // Ensure that all of the cached values are available
        self.fill_cache();

        // Fetch the regions and the paths, ready for rendering
        let regions = self.get_cached_regions();

        // Process the regions to generate the final rendering
        self.cache.sync(move |cache| {
            cache.render_at_time(time, &*regions, gc);
        });
    }

    ///
    /// Returns a graphics context for this layer
    ///
    pub fn graphics_context<'a>(&'a mut self) -> AnimationLayerContext<'a> {
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

        AnimationLayerContext {
            animation_layer:    self,
            cache:              vec![]
        }
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

impl<'a> AnimationLayerContext<'a> {
    ///
    /// Adds a drawing instruction to the context (calling this method 'push' lets us copy the implementation from flo_draw's Vec<Draw> so it's easy to keep up to date)
    ///
    #[inline]
    fn push(&mut self, drawing: Draw) {
        // Cache the drawing instructions
        self.cache.push(drawing);

        // Send to the animation layer once the cache has built up enough
        if self.cache.len() > 128 {
            self.animation_layer.draw(self.cache.drain(..));
        }
    }

    ///
    /// Adds an animation region to the layer that this context is for
    ///
    pub fn add_region<Region: 'static+AnimationRegion>(&mut self, region: Region) {
        // Flush the cache to the current time
        if self.cache.len() > 0 {
            self.animation_layer.draw(self.cache.drain(..));
        }

        // Add the region
        self.animation_layer.add_region(region);
    }

    ///
    /// Updates the time where the current set of drawing will be rendered
    ///
    #[inline]
    pub fn set_time(&mut self, time: Duration) {
        // Flush the cache to the current time
        if self.cache.len() > 0 {
            self.animation_layer.draw(self.cache.drain(..));
        }

        // Set the time for future drawing instructions
        self.animation_layer.set_time(time);
    }
}

impl<'a> Drop for AnimationLayerContext<'a> {
    fn drop(&mut self) {
        self.animation_layer.draw(self.cache.drain(..));
        self.animation_layer.cache.desync(|cache| cache.flush());
    }
}

impl<'a> GraphicsContext for AnimationLayerContext<'a> {
    #[inline]
    fn draw(&mut self, d: Draw) {
        self.push(d);
    }
}
