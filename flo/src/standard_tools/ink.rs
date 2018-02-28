use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use ui::*;
use binding::*;
use animation::*;

use futures::*;
use std::sync::*;

///
/// Data for the ink brush
/// 
#[derive(Clone, PartialEq, Debug)]
pub struct InkData {
    pub brush:              BrushDefinition,
    pub brush_properties:   BrushProperties,
    pub selected_layer:     u64
}

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
}

impl<Anim: Animation+'static> Tool<InkData, Anim> for Ink {
    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn menu_controller_name(&self) -> String { INKMENUCONTROLLER.to_string() }

    fn actions_for_model(&self, model: Arc<FloModel<Anim>>) -> Box<Stream<Item=ToolAction<InkData>, Error=()>+Send> {
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

    fn actions_for_input<'a>(&'a self, _data: Option<Arc<InkData>>, input: Box<'a+Iterator<Item=ToolInput<InkData>>>) -> Box<'a+Iterator<Item=ToolAction<InkData>>> {
        use self::ToolInput::*;
        use self::ToolAction::*;
        use self::BrushPreviewAction::*;

        let actions = input.flat_map(|input| {
            match input {
                ToolInput::Data(ref ink_data)   => vec![
                    // Set the brush preview status
                    BrushPreview(Clear),                // Clear on whatever layer the preview is currently on
                    BrushPreview(Layer(ink_data.selected_layer)),
                    BrushPreview(Clear),                // Clear on the new layer
                    BrushPreview(BrushDefinition(ink_data.brush.clone(), BrushDrawingStyle::Draw)),
                    BrushPreview(BrushProperties(ink_data.brush_properties.clone()))
                ],

                PaintDevice(_device)            => vec![
                    // Switching devices clears any preview
                    BrushPreview(Clear)
                ],

                Paint(painting)                 => {
                    match painting.action {
                        PaintAction::Start      => vec![
                            // Starting a new brush stroke starts a new brush preview
                            BrushPreview(Clear),
                            BrushPreview(AddPoint(raw_point_from_painting(&painting)))
                        ],
                        
                        PaintAction::Continue   => vec![
                            // Adds another point to the current brush stroke
                            BrushPreview(AddPoint(raw_point_from_painting(&painting)))
                        ],
                        
                        PaintAction::Finish     => vec![
                            // Brush stroke is finished: we commit it (committing also clears the preview)
                            BrushPreview(Commit)
                        ],

                        PaintAction::Cancel     => vec![
                            // Brush stroke canceled
                            BrushPreview(Clear)
                        ]
                    }
                }
            }.into_iter()
        });

        Box::new(actions)
    }
}
