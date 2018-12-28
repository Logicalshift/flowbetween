use super::super::tools::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::raycast::*;
use flo_curves::bezier::path::algorithms::*;

use itertools::*;

use std::iter;
use std::sync::*;

///
/// A tool for flood-filling areas of the canvas
///
pub struct FloodFill {

}

impl FloodFill {
    ///
    /// Creates a new flood-fill tool
    ///
    pub fn new() -> FloodFill {
        FloodFill {
        }
    }

    ///
    /// Generates the actions for a flood fill operation
    ///
    pub fn flood_fill<Anim: 'static+Animation>(&self, model: Arc<FloModel<Anim>>, center_point: (f32, f32)) -> impl Iterator<Item=ToolAction<()>> {
        // Turn the x, y coordinates into a pathpoint
        let (x, y)          = center_point;
        let center_point    = PathPoint::new(x, y);

        // Get the current frame information
        let when            = model.timeline().current_time.get();
        let layer           = model.timeline().selected_layer.get();
        let frame           = model.frame().frame.get();

        if let (Some(frame), Some(layer)) = (frame, layer) {
            // Generate a ray-casting function from it
            let ray_casting_fn  = vector_frame_raycast(&frame);

            // Attempt to generate a path element by flood-filling
            let fill_path       = flood_fill_convex(center_point, &FillOptions::default(), ray_casting_fn);

            if let Some(fill_path) = fill_path {
                // Create a new path element for this fill path
                let fill_path: Path = fill_path;

                let brush_defn      = BrushDefinition::Ink(InkDefinition::default());
                let mut brush_props = BrushProperties::new();

                brush_props.color   = Color::Rgba(0.0, 0.5, 0.8, 1.0);

                // Generate the editing actions to create this fill path
                let actions         = vec![
                    PathEdit::SelectBrush(ElementId::Unassigned, brush_defn, BrushDrawingStyle::Draw),
                    PathEdit::BrushProperties(ElementId::Unassigned, brush_props),
                    PathEdit::CreatePath(ElementId::Unassigned, Arc::new(fill_path.elements().collect()))
                ];
                let actions = actions.into_iter()
                    .map(move |action| LayerEdit::Path(when, action))
                    .map(move |action| AnimationEdit::Layer(layer, action))
                    .map(|action| ToolAction::Edit(action));

                Either::Left(actions)
            } else {
                Either::Right(iter::empty())
            }
        } else {
            Either::Right(iter::empty())
        }
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for FloodFill {
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Flood Fill".to_string() }

    fn image_name(&self) -> String { "floodfill".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn Iterator<Item=ToolAction<()>>> {
        Box::new(
            input.flat_map(move |action| {
                let actions : Box<dyn Iterator<Item=ToolAction<()>>> =
                    match action {
                        ToolInput::Paint(painting) => {
                            match painting.action {
                                PaintAction::Finish => {
                                    // Perform the flood-fill action when the painting finishes
                                    Box::new(self.flood_fill(Arc::clone(&flo_model), painting.location))
                                },

                                _ => {
                                    // Nothing to do for other paint actions
                                    Box::new(vec![].into_iter())
                                }
                            }
                        },

                        _ => {
                            // No action for other kinds of input input
                            Box::new(vec![].into_iter())
                        }
                    };

                actions
            })
            .collect::<Vec<_>>()
            .into_iter()
        )
    }
}
