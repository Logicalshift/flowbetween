use super::ink::*;
use super::super::tools::*;
use super::super::viewmodel::*;

use ui::*;
use binding::*;
use animation::*;
use animation::brushes::*;

use typemap::*;
use futures::*;
use std::sync::*;

impl Key for Eraser { type Value = BrushPreview; }

///
/// The Eraser tool (Erasers control points of existing objects)
/// 
pub struct Eraser { 
    ink: Ink
}

impl Eraser {
    ///
    /// Creates a new instance of the Eraser tool
    /// 
    pub fn new() -> Eraser {
        Eraser {
            ink: Ink::new()
        }
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

impl<Anim: Animation+'static> Tool2<InkData, Anim> for Eraser {
    fn tool_name(&self) -> String { "Eraser".to_string() }

    fn image_name(&self) -> String { "eraser".to_string() }

    fn actions_for_model(&self, model: Arc<AnimationViewModel<Anim>>) -> Box<Stream<Item=ToolAction<InkData>, Error=()>> {
        // Fetch the brush properties
        let brush_properties    = model.brush().brush_properties.clone();
        let selected_layer      = model.timeline().selected_layer.clone();

        // Create a computed binding that generates the data for the brush
        let ink_data            = computed(move || {
            InkData {
                brush:              BrushDefinition::Ink(InkDefinition::default()),
                brush_properties:   brush_properties.get(),
                selected_layer:     selected_layer.get().unwrap_or(0)
            }
        });

        // Turn the computed values into a stream and update the brush whenever the values change
        Box::new(follow(ink_data).map(|ink_data| ToolAction::Data(ink_data)))
    }

    fn actions_for_input<'a>(&'a self, data: Option<Arc<InkData>>, input: Box<'a+Iterator<Item=ToolInput<InkData>>>) -> Box<'a+Iterator<Item=ToolAction<InkData>>> {
        use self::ToolAction::*;
        use self::BrushPreviewAction::*;

        let ink: &Tool2<InkData, Anim> = &self.ink;

        // As for the ink tool, except that we use the eraser drawing style
        let actions = ink.actions_for_input(data, input)
            .map(|action| {
                match action {
                    BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Draw)) => BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Erase)),
                    
                    other => other
                }
            });

        Box::new(actions)
    }
}
