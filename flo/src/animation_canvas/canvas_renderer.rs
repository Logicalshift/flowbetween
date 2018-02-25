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
    layer_frame:    Arc<Frame>,

    /// The brush that was last used for this layer
    active_brush: Option<(BrushDefinition, BrushDrawingStyle)>,

    /// The brush properties that were last used for this layer
    active_properties: Option<BrushProperties>
}

///
/// Performs rendering of a canvas
/// 
pub struct CanvasRenderer {
    /// The layers in the current frame
    frame_layers: HashMap<u64, FrameLayer>,

    /// The layer that we're currently 'annotating'
    annotated_layer: Option<u64>
}

impl CanvasRenderer {
    ///
    /// Creates a new canvas renderer
    /// 
    pub fn new() -> CanvasRenderer {
        CanvasRenderer {
            frame_layers:       HashMap::new(),
            annotated_layer:    None
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
        let layer_frame             = layer.get_frame_at_time(time);
        let active_brush            = layer_frame.active_brush();
        let active_brush_properties = layer_frame.active_brush_properties();

        // Store this layer in the hashmap with its layer ID
        self.frame_layers.insert(layer.id(), FrameLayer {
            layer_id:           layer_id,
            layer_frame:        layer_frame,
            active_brush:       active_brush,
            active_properties:  active_brush_properties
        });
    }

    ///
    /// Clears a canvas and sets it up for rendering
    /// 
    fn clear_canvas(&mut self, canvas: &BindingCanvas, (width, height): (f64, f64)) {
        // Clearing the canvas also removes any 'annotations' that might have been performed
        self.annotated_layer = None;

        canvas.draw(move |gc| {
            gc.clear_canvas();
            gc.canvas_height((height*1.05) as f32);
            gc.center_region(0.0,0.0, width as f32, height as f32);
        });
    }

    ///
    /// Draws the canvas background to a context
    /// 
    fn draw_background(&mut self, gc: &mut GraphicsPrimitives, (width, height): (f64, f64)) {
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
    pub fn draw_frame_layers(&mut self, canvas: &BindingCanvas, size: (f64, f64)) {
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

    ///
    /// Ensures that any annotations that have been performed are cleared
    /// 
    pub fn clear_annotation(&mut self, canvas: &BindingCanvas) {
        // Fetch & clear the currently annotated layer
        let annotated_layer = self.annotated_layer.take();

        // If the annotated layer exists, then restore the canvas
        if annotated_layer.is_some() {
            canvas.draw(|gc| gc.restore());
        }
    }

    ///
    /// Given a layer ID, draws an annotation on top (replacing any existing annotation)
    /// 
    /// The annotation is just a drawing that is on top of the 'real' layer drawing
    /// and can be replaced at any time. This allows for drawing things like preview
    /// brush strokes without needing to redraw the entire canvas.
    /// 
    pub fn annotate_layer<DrawFn: FnOnce(&mut GraphicsPrimitives) -> ()+Send>(&mut self, canvas: &BindingCanvas, layer_id: u64, draw_annotations: DrawFn) {
        let previous_layer = self.annotated_layer;

        // We can't currently have annotations on more than one layer at once (this is because 'restore' does not function
        // correctly if the active layer is changed)

        // The existing annotation is cleared by this action
        self.clear_annotation(canvas);

        // Attempt to retrieve the canvas layer ID for the animation layer
        let canvas_layer_id = self.frame_layers.get(&layer_id).map(|frame_layer| frame_layer.layer_id);

        // Annotation is drawn if we can find the frame layer for the layer we're annotating
        if let Some(canvas_layer_id) = canvas_layer_id {
            // This is now the annotated layer
            self.annotated_layer = Some(layer_id);

            // Render the canvas
            canvas.draw(move |gc| {
                // If the layer being annotated has changed, then we need to switch layers
                if Some(layer_id) != previous_layer {
                    // Throw away the stored buffer here
                    gc.free_stored_buffer();

                    // Set the layer and store the backing buffer
                    gc.layer(canvas_layer_id);
                    gc.store();
                }

                // Draw the annotations
                draw_annotations(gc);
            });
        }
    }

    ///
    /// Removes any annotation and then commits some drawing actions to a particular layer
    /// 
    /// In general this is useful at the end of a brush stroke, where we want to finalize
    /// the results of a drawing without having to redraw the entire layer.
    /// 
    pub fn commit_to_layer<DrawFn: FnOnce(&mut GraphicsPrimitives) -> ()+Send>(&mut self, canvas: &BindingCanvas, layer_id: u64, commit_drawing: DrawFn) {
        // The currently annotated layer will be selected, so we can elide the layer select command if it's the same layer the user wants to commit drawing to
        let previous_layer = self.annotated_layer;

        // If there's an annotation, clear it and remove any buffer that might be present
        if self.annotated_layer.is_some() {
            self.clear_annotation(canvas);
            canvas.draw(|gc| gc.free_stored_buffer());
        }

        // Attempt to retrieve the canvas layer ID for the animation layer
        let canvas_layer_id = self.frame_layers.get(&layer_id).map(|frame_layer| frame_layer.layer_id);

        if let Some(canvas_layer_id) = canvas_layer_id {
            canvas.draw(move |gc| {
                // Set the layer if it has changed
                if previous_layer != Some(layer_id) {
                    gc.layer(canvas_layer_id);
                }

                // Commit the requested drawing operations
                commit_drawing(gc);
            });
        }
    }

    ///
    /// Retrieves the brush settings for the specified layer 
    /// 
    pub fn get_layer_brush(&self, layer_id: u64) -> (Option<(BrushDefinition, BrushDrawingStyle)>, Option<BrushProperties>) {
        if let Some(layer) = self.frame_layers.get(&layer_id) {
            (layer.active_brush.clone(), layer.active_properties.clone())
        } else {
            (None, None)
        }
    }

    ///
    /// Sets the layer brush for the specified layer (eg after committing a brush preview)
    /// 
    pub fn set_layer_brush(&mut self, layer_id: u64, brush: Option<(BrushDefinition, BrushDrawingStyle)>, properties: Option<BrushProperties>) {
        if let Some(layer) = self.frame_layers.get_mut(&layer_id) {
            layer.active_brush      = brush;
            layer.active_properties = properties;
        }
    }
}
