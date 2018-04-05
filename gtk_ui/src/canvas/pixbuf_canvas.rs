use super::viewport::*;
use super::cairo_draw::*;

use flo_canvas::*;

use cairo;

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
    current_layer: u32
}

impl PixBufCanvas {
    ///
    /// Creates a new pixbuf canvas
    /// 
    pub fn new(viewport: CanvasViewport) -> PixBufCanvas {
        PixBufCanvas {
            layers:         HashMap::new(),
            viewport:       viewport,
            current_layer:  0
        }
    }

    ///
    /// Performs a drawing action on this canvas
    /// 
    pub fn draw(&mut self, action: Draw) {
        // Clearing the canvas clears all the layers and resets us to layer 0
        if action == Draw::ClearCanvas {
            self.layers.clear();
            self.current_layer = 0;
        }

        let current_layer = self.current_layer;
        
        // TODO: switching layers should bring over the state settings from the previous layer
        // TODO: storing/restoring parts of a layer

        // The current layer must exist
        if !self.layers.contains_key(&current_layer) {
            self.create_layer(current_layer);
        }

        // Fetch the layer we're going to draw to
        let layer = self.layers.get_mut(&self.current_layer);

        // Perform drawing
        match action {
            Draw::Layer(new_layer)  => self.current_layer = new_layer,
            
            // Other actions go to the current layer
            other_action            => { layer.map(|layer| layer.context.draw(other_action)); }
        }
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
    /// Creates a new layer
    /// 
    fn create_layer(&mut self, layer_id: u32) {
        let width   = self.viewport.viewport_width;
        let height  = self.viewport.viewport_height;

        // Perform the incantations to create a pixbuf we can draw on
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        // Pass on to a new CairoDraw instance
        let draw    = CairoDraw::new(context, self.viewport);

        // Store as a new layer
        let new_layer = Layer {
            surface: surface,
            context: draw
        };

        self.layers.insert(layer_id, new_layer);
    }
}