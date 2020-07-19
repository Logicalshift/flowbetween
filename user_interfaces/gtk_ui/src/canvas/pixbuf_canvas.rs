use super::viewport::*;
use super::cairo_draw::*;

use flo_canvas::*;

use cairo;
use cairo::*;

use std::collections::HashMap;

struct Layer {
    /// The surface this layer is drawn upon
    surface: cairo::ImageSurface,

    /// Stored variation of this surface
    stored: Option<cairo::ImageSurface>,

    /// Context that this surface will be drawn upon
    context: CairoDraw
}

///
/// The pixbuf canvas performs drawing operations using GDK pixel buffers for layers using Cairo
///
pub struct PixBufCanvas {
    /// The layers in this canvas
    layers: HashMap<u32, Layer>,

    /// The pixel scale for this canvas
    pixel_scale: f64,

    /// The viewport for this canvas
    viewport: CanvasViewport,

    /// The currently selected layer
    current_layer: u32,

    /// The state to restore during the next drawing operation
    saved_state: Option<CairoState>
}

impl PixBufCanvas {
    ///
    /// Creates a new pixbuf canvas
    ///
    pub fn new(viewport: CanvasViewport, pixel_scale: f64) -> PixBufCanvas {
        PixBufCanvas {
            layers:         HashMap::new(),
            pixel_scale:    pixel_scale,
            viewport:       viewport,
            current_layer:  0,
            saved_state:    None
        }
    }

    ///
    /// Sets the pixel scaling factor to be used for future layers
    ///
    pub fn set_pixel_scale(&mut self, new_scale: f64) {
        self.pixel_scale = new_scale;
    }

    ///
    /// Generates a stored version of the specified layer
    ///
    fn save_layer(&mut self, layer_id: u32) {
        let width           = self.viewport.viewport_width;
        let height          = self.viewport.viewport_height;
        let viewport        = &self.viewport;

        // Get or create the layer we're saving (we'll save an empty layer if it's new)
        let pixel_scale     = self.pixel_scale;
        let layer           = self.layers.entry(layer_id).or_insert_with(|| Self::create_layer(viewport, pixel_scale));

        // Remove any stored value from the layer
        layer.stored = None;

        // Create a new stored context to draw on
        let stored_surface  = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let stored_context  = cairo::Context::new(&stored_surface);

        let surface_pattern = cairo::SurfacePattern::create(&layer.surface);
        surface_pattern.set_filter(cairo::Filter::Nearest);

        // Copy from the layer surface to our stored surface
        stored_context.set_source(&surface_pattern);
        stored_context.set_antialias(cairo::Antialias::None);
        stored_context.set_operator(cairo::Operator::Source);
        stored_context.paint();

        // Store in the layer
        layer.stored = Some(stored_surface);
    }

    ///
    /// Restores the specified layer from its stored version
    ///
    fn restore_layer(&mut self, layer_id: u32) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            // Get the layer surface
            let layer_surface = &layer.surface;

            // Fetch the saved layer
            if let Some(saved_layer) = layer.stored.as_ref() {
                // Draw from the saved layer onto the main layer
                let layer_context = cairo::Context::new(&layer_surface);

                let surface_pattern = cairo::SurfacePattern::create(&saved_layer);
                surface_pattern.set_filter(cairo::Filter::Nearest);

                layer_context.set_source(&surface_pattern);
                layer_context.set_antialias(cairo::Antialias::None);
                layer_context.set_operator(cairo::Operator::Source);
                layer_context.paint();
            }
        }
    }

    ///
    /// Clears the storage associated with a layer
    ///
    fn clear_storage(&mut self, layer_id: u32) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.stored = None;
        }
    }

    ///
    /// Performs a drawing action on this canvas
    ///
    pub fn draw(&mut self, action: Draw) {
        match action {
            Draw::ClearCanvas => {
                // Clearing the canvas clears all the layers and resets us to layer 0
                self.layers.clear();
                self.saved_state    = None;
                self.current_layer  = 0;
            },

            Draw::ClearLayer => {
                let current_layer   = self.current_layer;
                let viewport        = &self.viewport;

                // Save the state so that it's preserved when the layer restarts
                if self.saved_state.is_none() {
                    // If there is already a saved state, we don't save from the new layer (presumably no operations have been performed so the older state is the one that should be kept)
                    self.saved_state        = self.layers.get(&current_layer).map(|layer| layer.context.get_state());
                }

                // Send the clear request to the current layer
                let pixel_scale = self.pixel_scale;
                let layer       = self.layers.entry(current_layer).or_insert_with(|| Self::create_layer(viewport, pixel_scale));
                layer.context.draw(Draw::ClearLayer);
            },

            Draw::Layer(new_layer_id) => {
                // Save the state from the current layer if necessary
                if self.saved_state.is_none() {
                    // If there is already a saved state, we don't save from the new layer (presumably no operations have been performed so the older state is the one that should be kept)
                    let previous_layer_id   = self.current_layer;
                    self.saved_state        = self.layers.get(&previous_layer_id).map(|layer| layer.context.get_state());
                }

                // Changing the current layer sets which layer is selected
                self.current_layer = new_layer_id;
            },

            Draw::Store             => { let current_layer = self.current_layer; self.save_layer(current_layer); },
            Draw::Restore           => { let current_layer = self.current_layer; self.restore_layer(current_layer); },
            Draw::FreeStoredBuffer  => { let current_layer = self.current_layer; self.clear_storage(current_layer); },

            other_action => {
                // Fetch the current layer
                let current_layer   = self.current_layer;
                let viewport        = &self.viewport;
                let pixel_scale     = self.pixel_scale;
                let layer           = self.layers.entry(current_layer).or_insert_with(|| Self::create_layer(viewport, pixel_scale));

                // Restore the saved state if there is one
                if let Some(state) = self.saved_state.take() {
                    layer.context.set_state(&state);
                }

                // Draw on this layer's context
                layer.context.draw(other_action);
            }
        }
    }

    ///
    /// Retrieves the transformation matrix for this canvas
    ///
    pub fn get_matrix(&self) -> cairo::Matrix {
        let current_layer = self.current_layer;

        self.saved_state
            .as_ref()
            .map(|state| Some(state.get_matrix()))
            .unwrap_or_else(|| self.layers.get(&current_layer).map(|layer| layer.context.get_matrix()))
            .unwrap_or_else(|| cairo::Matrix::identity())
    }

    ///
    /// Renders the canvas to a particular drawable
    ///
    pub fn render_to_context(&self, drawable: &cairo::Context) {
        drawable.save();

        // Put the layers in order
        let mut layers: Vec<_> = self.layers.iter().collect();
        layers.sort_by(|&a, &b| a.0.cmp(b.0));

        drawable.set_antialias(cairo::Antialias::None);

        // Draw them to the target
        for (_, layer) in layers {
            let layer_pattern = cairo::SurfacePattern::create(&layer.surface);
            layer_pattern.set_filter(cairo::Filter::Nearest);

            drawable.set_operator(cairo::Operator::Over);
            drawable.set_source(&layer_pattern);
            drawable.paint();
        }

        drawable.restore();
    }

    ///
    /// Changes the viewport of this pixbuf (which also erases any existing drawing)
    ///
    pub fn set_viewport(&mut self, new_viewport: CanvasViewport) {
        self.layers.clear();
        self.viewport = new_viewport;
    }

    ///
    /// Retrieves the viewport for this pixbuf
    ///
    pub fn get_viewport(&self) -> CanvasViewport {
        self.viewport
    }

    ///
    /// Creates a new layer
    ///
    fn create_layer(viewport: &CanvasViewport, pixel_scale: f64) -> Layer {
        let width   = viewport.viewport_width;
        let height  = viewport.viewport_height;

        // Perform the incantations to create a pixbuf we can draw on
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        context.set_antialias(cairo::Antialias::Fast);

        // Pass on to a new CairoDraw instance
        let draw    = CairoDraw::new(context, *viewport, pixel_scale);

        // Store as a new layer
        let new_layer = Layer {
            surface:    surface,
            context:    draw,
            stored:     None
        };

        new_layer
    }
}
