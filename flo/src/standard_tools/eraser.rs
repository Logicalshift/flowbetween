use super::ink::*;
use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use futures::*;
use futures::stream::{BoxStream};
use std::sync::*;

///
/// TODO: really, we should make the eraser subtract from existing paths rather
/// than drawing over the top (this means when moving things around, any erasings
/// stick around: also when something is entire erased it should be removed from
/// the drawing).
///
/// We need to add path arithmetic at least before this is possible to do,
/// however.
///

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

impl<Anim: Animation+'static> Tool<Anim> for Eraser {
    type ToolData   = InkData;
    type Model      = InkModel;

    fn tool_name(&self) -> String { "Eraser".to_string() }

    fn image_name(&self) -> String { "eraser".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> InkModel {
        let model = InkModel::new();

        model.size.set(10.0);

        model
    }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(EraserMenuController::new(&tool_model.size, &tool_model.opacity)))
    }

    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> BoxStream<'static, ToolAction<InkData>> {
        // Fetch the brush properties
        let brush_properties    = tool_model.brush_properties.clone();
        let selected_layer      = flo_model.timeline().selected_layer.clone();

        // Create a computed binding that generates the data for the brush
        let ink_data            = computed(move || {
            InkData {
                brush:              BrushDefinition::Ink(InkDefinition::default()),
                brush_properties:   brush_properties.get(),
                selected_layer:     selected_layer.get().unwrap_or(0),
                modification_mode:  BrushModificationMode::Individual,
                representation:     BrushRepresentation::BrushStroke
            }
        });

        // Turn the computed values into a stream and update the brush whenever the values change
        Box::pin(follow(ink_data).map(|ink_data| ToolAction::Data(ink_data)))
    }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<InkData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<InkData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<InkData>>> {
        use self::ToolAction::*;
        use self::BrushPreviewAction::*;

        let ink: &dyn Tool<Anim, ToolData=InkData, Model=InkModel> = &self.ink;

        // As for the ink tool, except that we use the eraser drawing style
        let actions = ink.actions_for_input(flo_model, data, input)
            .map(|action| {
                match action {
                    BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Draw)) => BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Erase)),

                    other => other
                }
            });

        Box::new(actions)
    }
}
