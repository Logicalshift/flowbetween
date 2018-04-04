use super::viewport::*;
use super::cairo_draw::*;

use flo_canvas::*;

use gdk_pixbuf;
use gdk_pixbuf::Pixbuf;
use gdk_pixbuf::prelude::*;
use gdk::prelude::*;
use cairo;

use std::collections::HashMap;

struct Layer {
    pix_buf: Pixbuf,
    context: CairoDraw
}

///
/// The pixbuf canvas performs drawing operations using GDK pixel buffers for layers using Cairo
/// 
pub struct PixBufCanvas {
    /// The layers in this canvas
    layers: HashMap<u32, Layer>,

    /// The viewport for this canvas
    viewport: CanvasViewport
}

impl PixBufCanvas {
    ///
    /// Creates a new pixbuf canvas
    /// 
    pub fn new(viewport: CanvasViewport) -> PixBufCanvas {
        PixBufCanvas {
            layers:     HashMap::new(),
            viewport:   viewport
        }
    }

    ///
    /// Creates a new layer
    /// 
    fn create_layer(&mut self, layer_id: u32) {
        let width   = self.viewport.viewport_width;
        let height  = self.viewport.viewport_height;

        // Perform the incantations to create a pixbuf we can draw on
        let buf     = Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, width, height);
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        context.set_source_pixbuf(&buf, 0.0, 0.0);

        // Pass on to a new CairoDraw instance
        let draw    = CairoDraw::new(context, self.viewport);

        // Store as a new layer
        let new_layer = Layer {
            pix_buf: buf,
            context: draw
        };

        self.layers.insert(layer_id, new_layer);
    }
}