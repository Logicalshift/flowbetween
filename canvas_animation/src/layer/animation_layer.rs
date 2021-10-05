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
    #[inline] fn start_frame(&mut self)                                                         { self.push(Draw::StartFrame); }
    #[inline] fn show_frame(&mut self)                                                          { self.push(Draw::ShowFrame); }
    #[inline] fn reset_frame(&mut self)                                                         { self.push(Draw::ResetFrame); }
    #[inline] fn new_path(&mut self)                                                            { self.push(Draw::Path(PathOp::NewPath)); }
    #[inline] fn move_to(&mut self, x: f32, y: f32)                                             { self.push(Draw::Path(PathOp::Move(x, y))); }
    #[inline] fn line_to(&mut self, x: f32, y: f32)                                             { self.push(Draw::Path(PathOp::Line(x, y))); }
    #[inline] fn bezier_curve_to(&mut self, x1: f32, y1: f32, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32) { 
        self.push(Draw::Path(PathOp::BezierCurve(((cp1x, cp1y), (cp2x, cp2y)), (x1, y1)))); 
    }
    #[inline] fn close_path(&mut self)                                                          { self.push(Draw::Path(PathOp::ClosePath)); }
    #[inline] fn fill(&mut self)                                                                { self.push(Draw::Fill); }
    #[inline] fn stroke(&mut self)                                                              { self.push(Draw::Stroke); }
    #[inline] fn line_width(&mut self, width: f32)                                              { self.push(Draw::LineWidth(width)); }
    #[inline] fn line_width_pixels(&mut self, width: f32)                                       { self.push(Draw::LineWidthPixels(width)); }
    #[inline] fn line_join(&mut self, join: LineJoin)                                           { self.push(Draw::LineJoin(join)); }
    #[inline] fn line_cap(&mut self, cap: LineCap)                                              { self.push(Draw::LineCap(cap)); }
    #[inline] fn winding_rule(&mut self, rule: WindingRule)                                     { self.push(Draw::WindingRule(rule)); }
    #[inline] fn new_dash_pattern(&mut self)                                                    { self.push(Draw::NewDashPattern); }
    #[inline] fn dash_length(&mut self, length: f32)                                            { self.push(Draw::DashLength(length)); }
    #[inline] fn dash_offset(&mut self, offset: f32)                                            { self.push(Draw::DashOffset(offset)); }
    #[inline] fn fill_color(&mut self, col: Color)                                              { self.push(Draw::FillColor(col)); }
    #[inline] fn fill_texture(&mut self, t: TextureId, x1: f32, y1: f32, x2: f32, y2: f32)      { self.push(Draw::FillTexture(t, (x1, y1), (x2, y2))); }
    #[inline] fn fill_gradient(&mut self, g: GradientId, x1: f32, y1: f32, x2: f32, y2: f32)    { self.push(Draw::FillGradient(g, (x1, y1), (x2, y2))); }
    #[inline] fn fill_transform(&mut self, transform: Transform2D)                              { self.push(Draw::FillTransform(transform)); }
    #[inline] fn stroke_color(&mut self, col: Color)                                            { self.push(Draw::StrokeColor(col)); }
    #[inline] fn blend_mode(&mut self, mode: BlendMode)                                         { self.push(Draw::BlendMode(mode)); }
    #[inline] fn identity_transform(&mut self)                                                  { self.push(Draw::IdentityTransform); }
    #[inline] fn canvas_height(&mut self, height: f32)                                          { self.push(Draw::CanvasHeight(height)); }
    #[inline] fn center_region(&mut self, minx: f32, miny: f32, maxx: f32, maxy: f32)           { self.push(Draw::CenterRegion((minx, miny), (maxx, maxy))); }
    #[inline] fn transform(&mut self, transform: Transform2D)                                   { self.push(Draw::MultiplyTransform(transform)); }
    #[inline] fn unclip(&mut self)                                                              { self.push(Draw::Unclip); }
    #[inline] fn clip(&mut self)                                                                { self.push(Draw::Clip); }
    #[inline] fn store(&mut self)                                                               { self.push(Draw::Store); }
    #[inline] fn restore(&mut self)                                                             { self.push(Draw::Restore); }
    #[inline] fn free_stored_buffer(&mut self)                                                  { self.push(Draw::FreeStoredBuffer); }
    #[inline] fn push_state(&mut self)                                                          { self.push(Draw::PushState); }
    #[inline] fn pop_state(&mut self)                                                           { self.push(Draw::PopState); }
    #[inline] fn clear_canvas(&mut self, color: Color)                                          { self.push(Draw::ClearCanvas(color)); }
    #[inline] fn layer(&mut self, layer_id: LayerId)                                            { self.push(Draw::Layer(layer_id)); }
    #[inline] fn layer_blend(&mut self, layer_id: LayerId, blend_mode: BlendMode)               { self.push(Draw::LayerBlend(layer_id, blend_mode)); }
    #[inline] fn clear_layer(&mut self)                                                         { self.push(Draw::ClearLayer); }
    #[inline] fn clear_all_layers(&mut self)                                                    { self.push(Draw::ClearAllLayers); }
    #[inline] fn swap_layers(&mut self, layer1: LayerId, layer2: LayerId)                       { self.push(Draw::SwapLayers(layer1, layer2)); }
    #[inline] fn sprite(&mut self, sprite_id: SpriteId)                                         { self.push(Draw::Sprite(sprite_id)); }
    #[inline] fn clear_sprite(&mut self)                                                        { self.push(Draw::ClearSprite); }
    #[inline] fn sprite_transform(&mut self, transform: SpriteTransform)                        { self.push(Draw::SpriteTransform(transform)); }
    #[inline] fn draw_sprite(&mut self, sprite_id: SpriteId)                                    { self.push(Draw::DrawSprite(sprite_id)); }

    #[inline] fn define_font_data(&mut self, font_id: FontId, font_data: Arc<CanvasFontFace>)                                   { self.push(Draw::Font(font_id, FontOp::UseFontDefinition(font_data))); }
    #[inline] fn set_font_size(&mut self, font_id: FontId, size: f32)                                                           { self.push(Draw::Font(font_id, FontOp::FontSize(size))); }
    #[inline] fn draw_text(&mut self, font_id: FontId, text: String, baseline_x: f32, baseline_y: f32)                          { self.push(Draw::DrawText(font_id, text, baseline_x, baseline_y)); }
    #[inline] fn draw_glyphs(&mut self, font_id: FontId, glyphs: Vec<GlyphPosition>)                                            { self.push(Draw::Font(font_id, FontOp::DrawGlyphs(glyphs))); }
    #[inline] fn begin_line_layout(&mut self, x: f32, y: f32, align: TextAlignment)                                             { self.push(Draw::BeginLineLayout(x, y, align)); }
    #[inline] fn layout_text(&mut self, font_id: FontId, text: String)                                                          { self.push(Draw::Font(font_id, FontOp::LayoutText(text))); }
    #[inline] fn draw_text_layout(&mut self)                                                                                    { self.push(Draw::DrawLaidOutText); }

    #[inline] fn create_texture(&mut self, texture_id: TextureId, w: u32, h: u32, format: TextureFormat)                        { self.push(Draw::Texture(texture_id, TextureOp::Create(w, h, format))); }
    #[inline] fn free_texture(&mut self, texture_id: TextureId)                                                                 { self.push(Draw::Texture(texture_id, TextureOp::Free)); }
    #[inline] fn set_texture_bytes(&mut self, texture_id: TextureId, x: u32, y: u32, w: u32, h: u32, bytes: Arc<Vec<u8>>)       { self.push(Draw::Texture(texture_id, TextureOp::SetBytes(x, y, w, h, bytes))); }
    #[inline] fn set_texture_fill_alpha(&mut self, texture_id: TextureId, alpha: f32)                                           { self.push(Draw::Texture(texture_id, TextureOp::FillTransparency(alpha))); }

    #[inline] fn create_gradient(&mut self, gradient_id: GradientId, initial_color: Color)                                      { self.push(Draw::Gradient(gradient_id, GradientOp::Create(initial_color))); }
    #[inline] fn gradient_stop(&mut self, gradient_id: GradientId, pos: f32, color: Color)                                      { self.push(Draw::Gradient(gradient_id, GradientOp::AddStop(pos, color))); }

    #[inline]
    fn draw(&mut self, d: Draw) {
        self.push(d);
    }
}
