use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;

use futures::*;
use std::sync::*;
use std::collections::HashSet;

///
/// The Select tool (Selects control points of existing objects)
/// 
pub struct Select { }

impl Select {
    ///
    /// Creates a new instance of the Select tool
    /// 
    pub fn new() -> Select {
        Select {}
    }

    ///
    /// Returns the list of commands to set up for drawing some selections
    /// 
    fn selection_drawing_settings() -> Vec<Draw> {
        vec![
            Draw::ClearCanvas,
            Draw::LineWidthPixels(1.0),
            Draw::StrokeColor(Color::Rgba(0.2, 0.8, 1.0, 1.0)),
            Draw::NewPath
        ]
    }

    ///
    /// Returns the drawing actions to highlight the specified element
    /// 
    fn highlight_for_selection(element: &Vector, properties: &VectorProperties) -> Vec<Draw> {
        // Get the paths for this element
        let paths = element.to_path(properties);
        if let Some(paths) = paths {
            // Retrieve the bounding box for each of the paths
            let bounds = paths.into_iter().map(|path| path.bounding_box());

            // Merge into a single bounding box
            let bounds = bounds.fold(Rect::empty(), |current, next| current.union(next));

            // Draw a rectangle around these bounds
            let mut bounds: Vec<Draw> = bounds.into();
            bounds.push(Draw::Stroke);

            bounds
        } else {
            // There are no paths for this element
            vec![]
        }
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for Select {
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    fn create_model(&self) -> () { }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Option<Arc<Controller>> {
        Some(Arc::new(SelectMenuController::new()))
    }

    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Box<Stream<Item=ToolAction<()>, Error=()>+Send> {
        // The set of currently selected elements
        let selected = flo_model.selection().selected_element.clone();
        let selected = computed(move || -> HashSet<_> { selected.get().into_iter().collect() });

        // Create a binding that works out the current frame
        let current_frame = computed(move || {
            // Get the layer ID and the frame layers
            let layer_id        = flo_model.timeline().selected_layer.get();
            let frame_layers    = flo_model.frame().layers.get();

            let frame           = frame_layers.into_iter().filter(|frame| Some(frame.layer_id) == layer_id).nth(0);

            frame.map(|frame| frame.frame.get()).unwrap_or(None)
        });

        // Follow it, and draw an overlay showing all the bounding boxes
        Box::new(follow(current_frame)
            .map(|current_frame| {
                if let Some(current_frame) = current_frame {
                    // Get the elements in the current frame
                    let elements    = current_frame.vector_elements().unwrap_or_else(|| Box::new(vec![].into_iter()));
                    
                    // Build up a vector of bounds
                    let mut selection   = vec![];
                    let mut properties  = VectorProperties::default();

                    for element in elements {
                        // Update the properties according to this element
                        element.update_properties(&mut properties);

                        // Draw the settings for this element
                        selection.extend(Self::highlight_for_selection(&element, &properties));
                    }
                    
                    // Create the overlay drawing
                    let overlay = Self::selection_drawing_settings().into_iter()
                        .chain(selection);

                    ToolAction::Overlay(OverlayAction::Draw(overlay.collect()))
                } else {
                    // Just clear the overlay
                    ToolAction::Overlay(OverlayAction::Clear)
                }
            }))
    }

    fn actions_for_input<'a>(&self, _data: Option<Arc<()>>, _input: Box<'a+Iterator<Item=ToolInput<()>>>) -> Box<Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
