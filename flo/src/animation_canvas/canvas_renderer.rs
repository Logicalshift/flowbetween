use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::collections::HashMap;

///
/// Represents a layer in the current frame
///
struct FrameLayer {
    /// The ID of the layer to draw on the canvas
    layer_id:           u32,

    /// The frame data for this layer
    layer_frame:        Arc<dyn Frame>,

    /// The brush that was last used for this layer
    active_brush:       Option<(BrushDefinition, BrushDrawingStyle)>,

    /// The brush properties that were last used for this layer
    active_properties:  Option<BrushProperties>
}

///
/// Represents a layer containing an overlay
///
struct OverlayLayer {
    /// How layers in the overlay map to layers in the canvas
    layers:         HashMap<u32, u32>,

    /// The layer that is currently active for this overlay layer
    active_layer:   u32,

    /// The drawing for this layer
    drawing:        Canvas
}

///
/// Performs rendering of a canvas
///
pub struct CanvasRenderer {
    /// The layers in the current frame
    frame_layers: HashMap<u64, FrameLayer>,

    /// The over layers in the current frame
    overlay_layers: HashMap<u32, OverlayLayer>,

    /// The layer that we're currently 'annotating'
    annotated_layer: Option<u64>
}

impl OverlayLayer {
    ///
    /// Creates a new, empty, overlay layer
    ///
    pub fn new() -> OverlayLayer {
        OverlayLayer {
            layers:         HashMap::new(),
            active_layer:   0,
            drawing:        Canvas::new()
        }
    }
}

impl CanvasRenderer {
    ///
    /// Creates a new canvas renderer
    ///
    pub fn new() -> CanvasRenderer {
        CanvasRenderer {
            frame_layers:       HashMap::new(),
            overlay_layers:     HashMap::new(),
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
    /// Returns the ID of the first free layer
    ///
    fn free_layer(&self) -> u32 {
        // Create an iterator of all the used layer IDs
        let used_layers = self.frame_layers.values().map(|layer| layer.layer_id)
            .chain(self.overlay_layers.values().flat_map(|overlay| overlay.layers.values().map(|layer_id| *layer_id)));

        // Find the highest
        let max_layer = used_layers.max();

        // Result is one more than the highest used layer
        max_layer.unwrap_or(0)+1
    }

    ///
    /// Invalidates the layers assigned to overlay canvases
    ///
    fn invalidate_overlay_layers(&mut self) {
        self.overlay_layers.values_mut()
            .for_each(|value| value.layers = HashMap::new());
    }

    ///
    /// Given a set of drawing actions for an overlay, relays them to the specified canvas
    ///
    /// Overlays can call 'Layer' themselves: one important action this performs is mapping layer IDs generated as part of the overlay
    /// into unique layer IDs on the canvas itself.
    ///
    fn relay_drawing_for_overlay<DrawIter: Iterator<Item=Draw>>(&mut self, overlay: u32, gc: &mut dyn GraphicsPrimitives, drawing: DrawIter) {
        // Find the first free layer in this object
        let mut free_layer = self.free_layer();

        // Function to generate the next free layer if we need one
        let mut next_free_layer = move || {
            let layer_id = free_layer;
            free_layer += 1;
            layer_id
        };

        // Get (or create) the layer map for this overlay
        // We'll generate new entries in this map if unknown layers are encountered
        let overlay = self.overlay_layers
            .entry(overlay)
            .or_insert_with(|| OverlayLayer::new());

        // Pick the currently active layer (allocate it if it doesn't exist)
        let mut active_layer = overlay.active_layer;
        let canvas_layer = *overlay.layers.entry(active_layer).or_insert_with(|| next_free_layer());
        gc.layer(canvas_layer);

        // Map the drawing actions to actions for the target canvas (map layers mainly)
        for draw in drawing {
            use self::Draw::*;

            match draw {
                ClearCanvas => {
                    // Clear all the layers instead
                    for layer in overlay.layers.values() {
                        gc.layer(*layer);
                        gc.clear_layer();
                    }

                    // Active layer resets back to 0
                    active_layer = 0;
                    let canvas_layer = *overlay.layers.entry(active_layer).or_insert_with(|| next_free_layer());
                    gc.layer(canvas_layer);
                },

                Layer(overlay_layer) => {
                    // Pick the layer from the canvas
                    let canvas_layer = *overlay.layers.entry(overlay_layer).or_insert_with(|| next_free_layer());
                    gc.layer(canvas_layer);

                    // This becomes the new active layer
                    active_layer = overlay_layer;
                },

                LayerBlend(overlay_layer, blend_style) => {
                    // Pick the layer from the canvas
                    let canvas_layer = *overlay.layers.entry(overlay_layer).or_insert_with(|| next_free_layer());
                    gc.layer_blend(canvas_layer, blend_style);
                },

                unchanged => gc.draw(unchanged)
            }
        }

        // Update the active layer in the overlay (so future drawing commands go back to the right layer)
        overlay.active_layer = active_layer;
    }

    ///
    /// Loads a particular frame from a layer into this renderer
    ///
    pub fn load_frame(&mut self, model: FrameLayerModel) {
        // Load the frame data (we don't necessarily form a binding here)
        let frame = model.frame.get();

        if let Some(frame) = frame {
            // If there are any overlays, they get invalidated when we add this frame
            self.invalidate_overlay_layers();

            // The layer ID comes from the number of layers we've currently got loaded (this layer will be rendered on top of all others)
            let animation_layer_id      = model.layer_id;
            let canvas_layer_id         = (self.frame_layers.len() as u32) + 1;

            // Get the frame for this time
            let layer_frame             = frame;

            // Store this layer in the hashmap with its layer ID
            self.frame_layers.insert(animation_layer_id, FrameLayer {
                layer_id:           canvas_layer_id,
                layer_frame:        layer_frame,
                active_brush:       None,
                active_properties:  None
            });
        }
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
    fn draw_background(&mut self, gc: &mut dyn GraphicsPrimitives, (width, height): (f64, f64)) {
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
    /// Redraws all of the overlay layers on a canvas
    ///
    /// Overlay operations will clear any annotation that might have been added.
    ///
    pub fn draw_overlays(&mut self, canvas: &BindingCanvas) {
        // Overlays screw with the annotation: make sure it's cleared
        self.clear_annotation(canvas);

        // Draw the overlays
        canvas.draw(|gc| {
            // Copy the IDs (the drawing relay requires a mutable borrow so we need a copy)
            let overlay_layer_ids: Vec<_> = self.overlay_layers.keys().cloned().collect();

            // Draw each overlay in turn via the relay function
            for layer_id in overlay_layer_ids {
                let drawing = self.overlay_layers[&layer_id].drawing.get_drawing();
                if drawing.len() > 0 {
                    self.relay_drawing_for_overlay(layer_id, gc, drawing.into_iter());
                }
            }
        })
    }

    ///
    /// Sends some drawing commands to an overlay
    ///
    /// Overlay operations will clear any annotation that might have been added.
    ///
    pub fn overlay(&mut self, canvas: &BindingCanvas, overlay: u32, drawing: Vec<Draw>) {
        // Overlays screw with the annotation: make sure it's cleared
        self.clear_annotation(canvas);

        // Copy the drawing into the overlay's canvas
        {
            let overlay = self.overlay_layers.entry(overlay)
                .or_insert_with(|| OverlayLayer::new());
            overlay.drawing.write(drawing.clone());
        }

        // Relay the drawing to the binding canvas
        if drawing.len() > 0 {
            canvas.draw(|gc| {
                self.relay_drawing_for_overlay(overlay, gc, drawing.into_iter())
            });
        }
    }

    ///
    /// Clears the overlays from a canvas
    ///
    /// Overlay operations will clear any annotation that might have been added.
    ///
    pub fn clear_overlays(&mut self, canvas: &BindingCanvas) {
        // Overlays screw with the annotation: make sure it's cleared
        self.clear_annotation(canvas);

        // Clear all of the overlay layers
        {
            let overlay_layer_ids = self.overlay_layers.values()
                .flat_map(|overlay| overlay.layers.values());

            canvas.write(overlay_layer_ids.flat_map(|layer_id| vec![
                Draw::Layer(*layer_id),
                Draw::ClearLayer
            ].into_iter()).collect());
        }

        // Delete the overlays
        self.overlay_layers = HashMap::new();
    }

    ///
    /// Ensures that any annotations that have been performed are cleared
    ///
    pub fn clear_annotation(&mut self, canvas: &BindingCanvas) {
        // Fetch & clear the currently annotated layer
        let annotated_layer = self.annotated_layer.take();

        // If the annotated layer exists, then restore the canvas
        if annotated_layer.is_some() {
            canvas.draw(|gc| { gc.pop_state(); gc.restore(); });
        }
    }

    ///
    /// Given a layer ID, draws an annotation on top (replacing any existing annotation)
    ///
    /// The annotation is just a drawing that is on top of the 'real' layer drawing
    /// and can be replaced at any time. This allows for drawing things like preview
    /// brush strokes without needing to redraw the entire canvas.
    ///
    pub fn annotate_layer<DrawFn: FnOnce(&mut dyn GraphicsPrimitives) -> ()+Send>(&mut self, canvas: &BindingCanvas, layer_id: u64, draw_annotations: DrawFn) {
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

                // Always push the state so it can be cleared when the annotations go away
                gc.push_state();

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
    pub fn commit_to_layer<DrawFn: FnOnce(&mut dyn GraphicsPrimitives) -> ()+Send>(&mut self, canvas: &BindingCanvas, layer_id: u64, commit_drawing: DrawFn) {
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
