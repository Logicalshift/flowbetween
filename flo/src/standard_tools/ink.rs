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
}

impl InkModel {
    ///
    /// Creates a new ink model with the default settings
    /// 
    pub fn new() -> InkModel {
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

    ///
    /// Retrieves the name of this tool
    /// 
    fn tool_name(&self) -> String { "Ink".to_string() }

    ///
    /// Retrieves the name of the image that is associated with this tool
    /// 
    fn image_name(&self) -> String { "ink".to_string() }

    ///
    /// Creates a new instance of the UI model for this tool
    /// 
    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> InkModel { 
        InkModel::new()
    }

    ///
    /// Creates the menu controller for this tool (or None if this tool has no menu controller)
    /// 
    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> Option<Arc<Controller>> {
        Some(Arc::new(InkMenuController::new(&tool_model.size, &tool_model.opacity, &tool_model.color)))
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    /// 
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> Box<Stream<Item=ToolAction<InkData>, Error=()>+Send> {
        // Fetch the brush properties
        let brush_properties    = tool_model.brush_properties.clone();
        let selected_layer      = flo_model.timeline().selected_layer.clone();

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

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    /// 
    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<InkData>>, input: Box<'a+Iterator<Item=ToolInput<InkData>>>) -> Box<'a+Iterator<Item=ToolAction<InkData>>> {
        use self::BrushPreviewAction::*;
        use self::ToolAction::*;
        use self::ToolInput::*;

        let actions = input.flat_map(|input| {
            match input {
                ToolInput::Select | ToolInput::Deselect => vec![],

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
                            BrushPreview(Commit),

                            // Painting new brush strokes clears the current selection
                            ClearSelection
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
