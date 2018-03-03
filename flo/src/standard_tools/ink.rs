use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use ui::*;
use canvas::*;
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
/// The ink UI model
/// 
pub struct InkModel {
    /// The size of the brush (pixels)
    pub size: Binding<f32>,

    /// The opacity of the brush (0-1)
    pub opacity: Binding<f32>,

    /// The colour of the brush (in general alpha should be left at 1.0 here)
    pub color: Binding<Color>,

    /// The brush properties for the current brush view model
    pub brush_properties: BindRef<BrushProperties>
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

    ///
    /// Creates brush properties from the model bindings
    /// 
    fn brush_properties(size: Binding<f32>, opacity: Binding<f32>, color: Binding<Color>) -> BindRef<BrushProperties> {
        let brush_properties = computed(move || {
            BrushProperties {
                size:       size.get(),
                opacity:    opacity.get(),
                color:      color.get()
            }
        });

        BindRef::from(brush_properties)
    }
}

impl<Anim: Animation+'static> Tool<Anim> for Ink {
    type ToolData   = InkData;
    type Model      = InkModel;

    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn create_model(&self) -> InkModel { 
        let size                = bind(5.0);
        let opacity             = bind(1.0);
        let color               = bind(Color::Rgba(0.0, 0.0, 0.0, 1.0));

        let brush_properties    = Self::brush_properties(size.clone(), opacity.clone(), color.clone());

        InkModel {
            size:               size,
            opacity:            opacity,
            color:              color,
            brush_properties:   brush_properties
        }
    }

    fn menu_controller_name(&self) -> String { INKMENUCONTROLLER.to_string() }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> Option<Box<Controller>> {
        Some(Box::new(InkMenuController::new(&tool_model.size, &tool_model.opacity, &tool_model.color)))
    }

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
