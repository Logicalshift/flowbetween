use ui::*;
use canvas::*;
use animation::*;

use std::time::Duration;
use std::sync::*;
use std::collections::*;

///
/// Represents a layer in the current frame
/// 
struct FrameLayer {
    /// The ID of the layer to draw on the canvas
    layer_id:       u32,

    /// The frame data for this layer
    layer_frame:    Arc<Frame>
}

///
/// Performs rendering of a canvas
/// 
pub struct CanvasRenderer {
    /// The layers in the current frame
    frame_layers: HashMap<u64, FrameLayer>
}

impl CanvasRenderer {
    ///
    /// Creates a new canvas renderer
    /// 
    pub fn new() -> CanvasRenderer {
        CanvasRenderer {
            frame_layers: HashMap::new()
        }
    }

    ///
    /// Clears all layers from this renderer
    /// 
    pub fn clear(&mut self) {
        self.frame_layers = HashMap::new();
    }

    ///
    /// Loads a particular frame from a layer into this renderer
    /// 
    pub fn load_frame(&mut self, layer: &Layer, time: Duration) {
        // The layer ID comes from the number of layers we've currently got loaded (this layer will be rendered on top of all others)
        let layer_id    = (self.frame_layers.len() as u32) + 1;

        // Get the frame for this time
        let layer_frame = layer.get_frame_at_time(time);

        // Store this layer in the hashmap with its layer ID
        self.frame_layers.insert(layer.id(), FrameLayer {
            layer_id:       layer_id,
            layer_frame:    layer_frame
        });
    }

    ///
    /// Clears a canvas and sets it up for rendering
    /// 
    fn clear_canvas(&self, canvas: &mut BindingCanvas, (width, height): (f64, f64)) {
        canvas.draw(move |gc| {
            gc.clear_canvas();
            gc.canvas_height((height*1.05) as f32);
            gc.center_region(0.0,0.0, width as f32, height as f32);
        });
    }

    ///
    /// Draws the canvas background to a context
    /// 
    fn draw_background(&self, gc: &mut GraphicsPrimitives, (width, height): (f64, f64)) {
        // Work out the width, height to draw the animation to draw
        let (width, height) = (width as f32, height as f32);
        
        // Background always goes on layer 0
        gc.layer(0);

        gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        gc.line_width_pixels(1.0);

        // Draw the shadow
        let offset = height * 0.015;

        gc.fill_color(Color::Rgba(0.1, 0.1, 0.1, 0.4));
        gc.new_path();
        gc.rect(0.0, 0.0-offset, width+offset, height);
        gc.fill();

        // Draw the canvas background
        gc.fill_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));
        gc.new_path();
        gc.rect(0.0, 0.0, width, height);
        gc.fill();
        gc.stroke();
    }

    ///
    /// Draws the current set of frame layers to the specified canvas
    /// 
    pub fn draw_frame_layers(&self, canvas: &mut BindingCanvas, size: (f64, f64)) {
        // Clear the canvas and redraw the background
        self.clear_canvas(canvas, size);
        canvas.draw(|gc| self.draw_background(gc, size));

        // Draw the active set of layers
        canvas.draw(move |gc| {
            // Draw the layers
            for layer in self.frame_layers.values() {
                gc.layer(layer.layer_id);
                layer.layer_frame.render_to(gc);
            }
        });
    }
}
