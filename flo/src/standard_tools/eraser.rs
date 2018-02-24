use super::ink::*;
use super::super::tools::*;

use ui::*;
use binding::*;
use animation::*;
use animation::brushes::*;

use typemap::*;

impl Key for Eraser { type Value = BrushPreview; }

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
        // Create the brush preview
        let mut brush_preview = BrushPreview::new();
        brush_preview.select_brush(&BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Erase);
        brush_preview.set_brush_properties(&model.anim_view_model.brush().brush_properties.get());

        // Store the preview in the state
        model.tool_state.lock().unwrap().insert::<Eraser>(brush_preview);

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
    }

    fn paint<'a>(&self, model: &ToolModel<'a, Anim>, _device: &PaintDevice, actions: &Vec<Painting>) {
        // Should be a brush preview in the state
        let tool_state      = model.tool_state.clone();
        let mut tool_state  = tool_state.lock().unwrap();
        let brush_preview   = tool_state.get_mut::<Eraser>().unwrap();

        // Perform the paint actions
        for action in actions {
            Ink::paint_action(model, brush_preview, action);
        }

        // The start action will set us up for rendering the preview by setting up a stored state
        // We render here so we don't render repeatedly when there are multiple actions
        if !brush_preview.is_finished() {
            model.canvas.draw(|gc| {
                // Re-render the current brush stroke
                gc.restore();
                gc.store();
                brush_preview.draw_current_brush_stroke(gc);
            });
        }
    }
}
