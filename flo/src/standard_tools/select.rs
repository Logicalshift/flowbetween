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
/// The model for the Select tool
/// 
pub struct SelectModel {
    /// Contains a pointer to the current frame
    pub frame: BindRef<Option<Arc<Frame>>>,

    /// Contains the bounding boxes of the elements in the current frame
    pub bounding_boxes: BindRef<Vec<(ElementId, Rect)>>
}

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
    type Model      = SelectModel;

    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> SelectModel {
        // Create a binding that works out the frame for the currently selected layer
        let selected_layer  = flo_model.timeline().selected_layer.clone();
        let frame_layers    = flo_model.frame().layers.clone();

        let current_frame   = computed(move || {
            // Get the layer ID and the frame layers
            let layer_id        = selected_layer.get();
            let frame_layers    = frame_layers.get();

            let frame           = frame_layers.into_iter().filter(|frame| Some(frame.layer_id) == layer_id).nth(0);

            frame.map(|frame| frame.frame.get()).unwrap_or(None)
        });

        // Create a binding that works out the bounding boxes of the elements in the current frame
        let frame           = current_frame.clone();

        let bounding_boxes  = computed(move || {
            // Fetch the current frame
            let frame = frame.get();

            if let Some(frame) = frame {
                // Get the elements in the current frame
                let elements            = frame.vector_elements().unwrap_or_else(|| Box::new(vec![].into_iter()));

                // We need to track the vector properties through all of the elements in the frame
                let mut properties      = VectorProperties::default();
                let mut bounding_boxes  = vec![];

                for element in elements {
                    // Update the properties
                    element.update_properties(&mut properties);

                    // Get the paths for this element
                    let paths = element.to_path(&properties).unwrap_or(vec![]);

                    // Turn into a bounding box
                    let bounds = paths.into_iter()
                        .map(|path| path.bounding_box())
                        .fold(Rect::empty(), |current, next| current.union(next));

                    // Add to the result
                    bounding_boxes.push((element.id(), bounds));
                }

                bounding_boxes
            } else {
                // No bounding boxes if there's no frame
                vec![]
            }
        });

        SelectModel {
            frame:          BindRef::new(&current_frame),
            bounding_boxes: BindRef::new(&bounding_boxes)
        }
    }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &SelectModel) -> Option<Arc<Controller>> {
        Some(Arc::new(SelectMenuController::new()))
    }

    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &SelectModel) -> Box<Stream<Item=ToolAction<()>, Error=()>+Send> {
        // The set of currently selected elements
        let selected_elements = flo_model.selection().selected_element.clone();
        let selected_elements = computed(move || -> HashSet<_> { selected_elements.get().into_iter().collect() });

        // Create a binding that works out the frame for the currently selected layer
        let current_frame = tool_model.frame.clone();

        // Follow it, and draw an overlay showing all the bounding boxes
        Box::new(follow(computed(move || (current_frame.get(), selected_elements.get())))
            .map(|(current_frame, selected_elements)| {
                if let Some(current_frame) = current_frame {
                    // Get the elements in the current frame
                    let elements        = current_frame.vector_elements().unwrap_or_else(|| Box::new(vec![].into_iter()));
                    
                    // Build up a vector of bounds
                    let mut selection   = vec![];
                    let mut properties  = VectorProperties::default();

                    for element in elements {
                        // Update the properties according to this element
                        element.update_properties(&mut properties);

                        // If the element is selected, draw a highlight around it
                        let element_id = element.id();
                        if element_id.is_assigned() && selected_elements.contains(&element_id) {
                            // Draw the settings for this element
                            selection.extend(Self::highlight_for_selection(&element, &properties));
                        }
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
