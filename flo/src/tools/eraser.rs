use super::*;

///
/// The Eraser tool (Erasers control points of existing objects)
/// 
pub struct Eraser { }

impl Eraser {
    ///
    /// Creates a new instance of the Eraser tool
    /// 
    pub fn new() -> Eraser {
        Eraser {}
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for Eraser {
    fn tool_name(&self) -> String { "Eraser".to_string() }

    fn image_name(&self) -> String { "eraser".to_string() }

    fn activate<'a>(&self, model: &ToolModel<'a, Anim>) -> BindRef<ToolActivationState> { 
        /*
        let selected_layer: Option<Editor<PaintLayer+'static>>  = model.selected_layer.edit();

        if let Some(mut selected_layer) = selected_layer {
            // Pick the ink brush in erase mode for the current layer
            selected_layer.select_brush(&BrushDefinition::Ink(InkDefinition::default_eraser()), BrushDrawingStyle::Erase);
        }

        // If the selected layer is different, we need re-activation
        let activated_layer_id  = model.anim_view_model.timeline().selected_layer.get();
        let selected_layer      = Binding::clone(&model.anim_view_model.timeline().selected_layer);
        BindRef::from(computed(move || {
            if activated_layer_id == selected_layer.get() {
                ToolActivationState::Activated
            } else {
                ToolActivationState::NeedsReactivation
            }
        }))
        */
    }

    fn paint<'a>(&self, model: &ToolModel<'a, Anim>, _device: &PaintDevice, actions: &Vec<Painting>) {
        /*
        let selected_layer: Option<Editor<PaintLayer+'static>>  = model.selected_layer.edit();

        // Perform the paint actions on the selected layer if we can
        if let Some(mut selected_layer) = selected_layer {
            for action in actions {
                Ink::paint_action(model, &mut *selected_layer, action);
            }

            // If there's a brush stroke waiting, render it
            // Starting a brush stroke selects the layer and creates a save state, which 
            // we assume is still present for the canvas (this is fragile!)
            if selected_layer.has_pending_brush_stroke() {
                let layer: &PaintLayer  = &*selected_layer;

                model.canvas.draw(|gc| {
                    // Re-render the current brush stroke
                    gc.restore();
                    gc.store();
                    layer.draw_current_brush_stroke(gc);
                });
            }
        }
        */
    }
}
