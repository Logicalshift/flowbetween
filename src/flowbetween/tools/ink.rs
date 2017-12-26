use super::*;

///
/// The Ink tool (Inks control points of existing objects)
/// 
pub struct Ink { }

impl Ink {
    ///
    /// Creates a new instance of the Ink tool
    /// 
    pub fn new() -> Ink {
        Ink {}
    }

    ///
    /// Performs a single painting action on the canvas
    /// 
    fn paint_action(&self, canvas: &Canvas, layer_id: u64, layer: &mut PaintLayer, action: &Painting) {
        /*
        // Get when this paint stroke is being made
        let current_time = self.anim_view_model.timeline().current_time.get();

        // Get the canvas layer ID
        let canvas_layer_id = self.core.sync(|core| core.frame_layers.get(&layer_id).map(|layer| layer.layer_id));
        let canvas_layer_id = canvas_layer_id.unwrap_or(1);

        canvas.draw(move |gc| {
            // Perform the action
            match action.action {
                PaintAction::Start       => {
                    // Select the layer and store the current image state
                    gc.layer(canvas_layer_id);
                    gc.store();

                    // Begin the brush stroke
                    layer.start_brush_stroke(current_time, BrushPoint::from(action));
                },

                PaintAction::Continue    => {
                    // Append to the brush stroke
                    layer.continue_brush_stroke(BrushPoint::from(action));
                },

                PaintAction::Finish      => {
                    // Draw the 'final' brush stroke
                    gc.restore();
                    layer.draw_current_brush_stroke(gc);

                    // Finish the brush stroke
                    layer.finish_brush_stroke();
                },

                PaintAction::Cancel      => {
                    // Cancel the brush stroke
                    layer.cancel_brush_stroke();
                    gc.restore();
                }
            }
        });
        */
    }
}

impl Tool for Ink {
    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn paint(&self, canvas: &Canvas, selected_layer: Arc<Layer>, _device: &PaintDevice, actions: &Vec<Painting>) {
        let layer_id                                            = selected_layer.id();
        let selected_layer: Option<Editor<PaintLayer+'static>>  = selected_layer.edit();

        // Perform the paint actions on the selected layer if we can
        if let Some(mut selected_layer) = selected_layer {
            for action in actions {
                self.paint_action(canvas, layer_id, &mut *selected_layer, action);
            }

            // If there's a brush stroke waiting, render it
            // Starting a brush stroke selects the layer and creates a save state, which 
            // we assume is still present for the canvas (this is fragile!)
            if selected_layer.has_pending_brush_stroke() {
                let layer: &PaintLayer  = &*selected_layer;

                canvas.draw(|gc| {
                    // Re-render the current brush stroke
                    gc.restore();
                    gc.store();
                    layer.draw_current_brush_stroke(gc);
                });
            }
        }
    }
}
