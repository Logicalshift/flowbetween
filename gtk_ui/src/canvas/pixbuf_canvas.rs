use super::viewport::*;
use super::cairo_draw::*;

use flo_canvas::*;

use cairo;
use cairo::prelude::*;

use std::collections::HashMap;

struct Layer {
    surface: cairo::ImageSurface,
    context: CairoDraw
}

///
/// The pixbuf canvas performs drawing operations using GDK pixel buffers for layers using Cairo
/// 
pub struct PixBufCanvas {
    /// The layers in this canvas
    layers: HashMap<u32, Layer>,

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
    pub fn new(viewport: CanvasViewport) -> PixBufCanvas {
        PixBufCanvas {
            layers:         HashMap::new(),
            viewport:       viewport,
            current_layer:  0,
            saved_state:    None
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

            other_action => {
                // Fetch the current layer
                let current_layer   = self.current_layer;
                let viewport        = &self.viewport;
                let layer           = self.layers.entry(current_layer).or_insert_with(|| Self::create_layer(viewport));

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
        // Put the layers in order
        let mut layers: Vec<_> = self.layers.iter().collect();
        layers.sort_by(|&a, &b| a.0.cmp(b.0));

        // Draw them to the target
        for (_, layer) in layers {
            drawable.set_operator(cairo::Operator::Over);
            drawable.set_source_surface(&layer.surface, 0.0, 0.0);
            drawable.paint();
        }
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
    fn create_layer(viewport: &CanvasViewport) -> Layer {
        let width   = viewport.viewport_width;
        let height  = viewport.viewport_height;

        // Perform the incantations to create a pixbuf we can draw on
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        // Pass on to a new CairoDraw instance
        let draw    = CairoDraw::new(context, *viewport);

        // Store as a new layer
        let new_layer = Layer {
            surface: surface,
            context: draw
        };

        new_layer
    }
}