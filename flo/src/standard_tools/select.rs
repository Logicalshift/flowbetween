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
#[derive(Clone)]
pub struct SelectModel {
    /// Contains a pointer to the current frame
    frame: BindRef<Option<Arc<Frame>>>,

    /// Contains the bounding boxes of the elements in the current frame
    bounding_boxes: BindRef<Arc<Vec<(ElementId, Rect)>>>
}

///
/// The actions that the tool can take
/// 
#[derive(Copy, Clone)]
enum SelectAction {
    /// No action (or the current action has been cancelled)
    NoAction,

    /// An item has been newly selected
    Select,

    /// The user is picking some items using a selection box
    RubberBand,

    /// The user has dragged their selection (either by selecting and moving away from the current location or by clicking on an item that's already selected)
    Drag
}

///
/// The select data provides feedback for the action being taken by the select tool
/// 
#[derive(Clone)]
pub struct SelectData {
    /// The current frame
    frame: Option<Arc<Frame>>,

    // The bounding boxes of the elements in the current frame
    bounding_boxes: Arc<Vec<(ElementId, Rect)>>,

    /// The current select action
    action: SelectAction,

    /// The position where the current action started
    initial_position: RawPoint
}

///
/// The Select tool (Selects control points of existing objects)
/// 
pub struct Select { }

impl SelectData {
    ///
    /// Creates a copy of this object with a different actions 
    ///
    fn with_action(&self, new_action: SelectAction) -> SelectData {
        SelectData {
            frame:              self.frame.clone(),
            bounding_boxes:     self.bounding_boxes.clone(),
            action:             new_action,
            initial_position:   self.initial_position.clone()
        }
    }
    
    ///
    /// Creates a copy of this object with a new initial position
    ///
    fn with_initial_position(&self, new_initial_position: RawPoint) -> SelectData {
        SelectData {
            frame:              self.frame.clone(),
            bounding_boxes:     self.bounding_boxes.clone(),
            action:             self.action,
            initial_position:   new_initial_position
        }
    }
}

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
            Draw::Layer(0),
            Draw::ClearLayer,

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

    ///
    /// Processes a paint action (at the top level)
    /// 
    fn paint(&self, paint: Painting, actions: Vec<ToolAction<SelectData>>, data: Arc<SelectData>) -> (Vec<ToolAction<SelectData>>, Arc<SelectData>) {
        (actions, data)
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for Select {
    type ToolData   = SelectData;
    type Model      = SelectModel;

    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    ///
    /// Creates the model for the Select tool
    /// 
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

                Arc::new(bounding_boxes)
            } else {
                // No bounding boxes if there's no frame
                Arc::new(vec![])
            }
        });

        SelectModel {
            frame:          BindRef::new(&current_frame),
            bounding_boxes: BindRef::new(&bounding_boxes)
        }
    }

    ///
    /// Creates the menu bar controller for the select tool
    /// 
    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &SelectModel) -> Option<Arc<Controller>> {
        Some(Arc::new(SelectMenuController::new()))
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    /// 
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &SelectModel) -> Box<Stream<Item=ToolAction<SelectData>, Error=()>+Send> {
        // The set of currently selected elements
        let selected_elements = flo_model.selection().selected_element.clone();
        let selected_elements = computed(move || -> HashSet<_> { selected_elements.get().into_iter().collect() });

        // Create a binding that works out the frame for the currently selected layer
        let current_frame = tool_model.frame.clone();

        // Follow it, and draw an overlay showing the bounding boxes of everything that's selected
        let draw_selection_overlay = follow(computed(move || (current_frame.get(), selected_elements.get())))
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
            });

        // Whenever the frame or the set of bounding boxes changes, we create a new SelectData object
        // (this also resets any in-progress action)
        let current_frame   = tool_model.frame.clone();
        let bounding_boxes  = tool_model.bounding_boxes.clone();
        let data_for_model  = follow(computed(move || (current_frame.get(), bounding_boxes.get())))
            .map(|(current_frame, bounding_boxes)| {
                ToolAction::Data(SelectData {
                    frame:              current_frame,
                    bounding_boxes:     bounding_boxes,
                    action:             SelectAction::NoAction,
                    initial_position:   RawPoint::from((0.0, 0.0))
                })
            });
        
        // Generate the final stream
        let select_stream = data_for_model.select(draw_selection_overlay);
        Box::new(select_stream)
    }

    ///
    /// Returns the actions that result from a particular inpiut
    /// 
    fn actions_for_input<'a>(&self, data: Option<Arc<SelectData>>, input: Box<'a+Iterator<Item=ToolInput<SelectData>>>) -> Box<Iterator<Item=ToolAction<SelectData>>> {
        if let Some(mut data) = data {
            // We build up a vector of actions to perform as we go
            let mut actions = vec![];

            // Process the inputs
            for input in input {
                match input {
                    ToolInput::Data(new_data) => {
                        // Whenever we get feedback about what the data is set to, update our data
                        data = new_data;
                    },

                    ToolInput::Select | ToolInput::Deselect => {
                        // Reset the action to 'no action' when the tool is selected or deselected
                        let new_data = data.with_action(SelectAction::NoAction);

                        // This replaces the data object
                        data = Arc::new(new_data.clone());

                        // And we get an action to update the data for the next set of inputs
                        actions.push(ToolAction::Data(new_data));
                    },

                    ToolInput::Paint(painting)  => {
                        let (new_actions, new_data) = self.paint(painting, actions, data);
                        actions = new_actions;
                        data    = new_data;
                    },

                    ToolInput::PaintDevice(_)   => ()
                }
            }

            // Return the actions that we built up
            Box::new(actions.into_iter())
        } else {
            // Received input before the tool is initialised
            Box::new(vec![].into_iter())
        }
    }
}
