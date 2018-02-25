use super::super::menu::*;
use super::super::tools::*;
use super::super::viewmodel::*;

use ui::*;
use binding::*;
use animation::*;
use animation::brushes::*;

use typemap::*;
use futures::*;
use std::sync::*;

impl Key for Ink { type Value = BrushPreview; }

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

    ///
    /// Performs a single painting action on the canvas
    /// 
    pub fn paint_action<'a, Anim: 'static+Animation>(model: &ToolModel<'a, Anim>, preview: &mut BrushPreview, action: &Painting) {
        // Get when this paint stroke is being made
        let current_time = model.current_time;

        // Get the canvas layer ID
        let canvas_layer_id     = model.canvas_layer_id;
        let selected_layer_id   = model.selected_layer_id;

        model.canvas.draw(move |gc| {
            // Perform the action
            match action.action {
                PaintAction::Start       => {
                    // Select the layer and store the current image state
                    gc.layer(canvas_layer_id);
                    gc.store();

                    // Begin the brush stroke
                    preview.update_brush_properties(&model.anim_view_model.brush().brush_properties.get());
                    preview.start_brush_stroke(raw_point_from_painting(action));
                },

                PaintAction::Continue    => {
                    // Append to the brush stroke
                    preview.continue_brush_stroke(raw_point_from_painting(action));
                },

                PaintAction::Finish      => {
                    // Draw the 'final' brush stroke
                    gc.restore();
                    preview.draw_current_brush_stroke(gc);

                    // Commit the brush stroke to the animation
                    preview.commit_to_animation(current_time, selected_layer_id, model.anim_view_model);
                },

                PaintAction::Cancel      => {
                    // Cancel the brush stroke
                    preview.cancel_brush_stroke();
                    gc.restore();
                }
            }
        });
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for Ink {
    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn menu_controller_name(&self) -> String { INKMENUCONTROLLER.to_string() }

    fn activate<'a>(&self, model: &ToolModel<'a, Anim>) -> BindRef<ToolActivationState> { 
        // Create the brush preview
        let mut brush_preview = BrushPreview::new();
        brush_preview.select_brush(&BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw);
        brush_preview.set_brush_properties(&model.anim_view_model.brush().brush_properties.get());

        // Store the preview in the state
        model.tool_state.lock().unwrap().insert::<Ink>(brush_preview);

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
        let brush_preview   = tool_state.get_mut::<Ink>().unwrap();

        // Perform the paint actions
        for action in actions {
            Self::paint_action(model, brush_preview, action);
        }

        if !brush_preview.is_finished() {
            // The start action will set us up for rendering the preview by setting up a stored state
            // We render here so we don't render repeatedly when there are multiple actions
            model.canvas.draw(|gc| {
                // Re-render the current brush stroke
                gc.restore();
                gc.store();
                brush_preview.draw_current_brush_stroke(gc);
            });
        }
    }
}

impl<Anim: Animation+'static> Tool2<InkData, Anim> for Ink {
    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn menu_controller_name(&self) -> String { INKMENUCONTROLLER.to_string() }

    fn actions_for_model(&self, model: Arc<AnimationViewModel<Anim>>) -> Box<Stream<Item=ToolAction<InkData>, Error=()>+Send> {
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
                            // Brush stroke is finished: we commit it
                            BrushPreview(Commit),
                            BrushPreview(Clear)
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
