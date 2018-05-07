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
    bounding_boxes: BindRef<Arc<Vec<(ElementId, Arc<VectorProperties>, Rect)>>>
}

///
/// The actions that the tool can take
/// 
#[derive(Copy, Clone, Debug)]
enum SelectAction {
    /// No action (or the current action has been cancelled)
    NoAction,

    /// An item has been newly selected
    Select,

    /// An item has been reselected
    Reselect,

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
    bounding_boxes: Arc<Vec<(ElementId, Arc<VectorProperties>, Rect)>>,

    // The current set of selected elements
    selected_elements: Arc<HashSet<ElementId>>,

    /// The current select action
    action: SelectAction,

    /// The position where the current action started
    initial_position: RawPoint,

    /// The position the user has dragged to
    drag_position: Option<RawPoint>
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
            selected_elements:  self.selected_elements.clone(),
            action:             new_action,
            initial_position:   self.initial_position.clone(),
            drag_position:      self.drag_position.clone()
        }
    }
    
    ///
    /// Creates a copy of this object with a new initial position
    ///
    fn with_initial_position(&self, new_initial_position: RawPoint) -> SelectData {
        SelectData {
            frame:              self.frame.clone(),
            bounding_boxes:     self.bounding_boxes.clone(),
            selected_elements:  self.selected_elements.clone(),
            action:             self.action,
            initial_position:   new_initial_position,
            drag_position:      None
        }
    }

    ///
    /// Creates a copy of this object with a new drag position
    /// 
    fn with_drag_position(&self, new_drag_position: RawPoint) -> SelectData {
        SelectData {
            frame:              self.frame.clone(),
            bounding_boxes:     self.bounding_boxes.clone(),
            selected_elements:  self.selected_elements.clone(),
            action:             self.action,
            initial_position:   self.initial_position.clone(),
            drag_position:      Some(new_drag_position)
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
    /// Returns the drawing instructions for drawing a rubber band around the specified points
    /// 
    fn draw_rubber_band(initial_point: (f32, f32), final_point: (f32, f32)) -> Vec<Draw> {
        // Create a bounding rectangle
        let bounds                  = Rect::with_points(initial_point.0, initial_point.1, final_point.0, final_point.1);
        let draw_bounds: Vec<Draw>  = bounds.normalize().into();

        // Setup actions
        let draw_setup = vec![
            Draw::Layer(1),
            Draw::ClearLayer,

            Draw::NewPath
        ];

        // Draw actions
        // Drawing an outer/inner section like this creates an effect that makes the highlight visible over nearly all backgrounds
        let draw_outer = vec![
            Draw::LineWidthPixels(2.0),
            Draw::StrokeColor(Color::Rgba(0.0, 0.0, 0.0, 0.1)),
            Draw::Stroke
        ];

        let draw_inner = vec![
            Draw::LineWidthPixels(0.5),
            Draw::StrokeColor(Color::Rgba(0.1, 0.7, 0.9, 1.0)),
            Draw::Stroke
        ];

        let draw_fill = vec![
            Draw::FillColor(Color::Rgba(0.6, 0.8, 0.9, 0.25)),
            Draw::Fill
        ];

        // Combine for the final result
        draw_setup.into_iter()
            .chain(draw_bounds)
            .chain(draw_fill)
            .chain(draw_outer)
            .chain(draw_inner)
            .collect()
    }

    ///
    /// Draws a set of dragged elements
    /// 
    fn draw_drag(data: &SelectData, selected_elements: Vec<(ElementId, Arc<VectorProperties>, Rect)>, initial_point: (f32, f32), drag_point: (f32, f32)) -> Vec<Draw> {
        let mut drawing = vec![];

        drawing.layer(1);
        drawing.clear_layer();

        // Draw everything translated by the drag distance
        drawing.push_state();
        drawing.transform(Transform2D::translate(drag_point.0-initial_point.0, drag_point.1-initial_point.1));

        // Draw the 'shadows' of the elements
        let mut bounding_boxes = vec![];
        for (element, properties, bounds) in selected_elements {
            // Update the brush properties to be a 'shadow' of the original
            let mut properties = (*properties).clone();
            properties.brush_properties.opacity *= 0.25;
            properties.brush_properties.color   = Color::Rgba(0.6, 0.8, 0.9, 1.0);

            // Fetch the element to render it
            let element = data.frame.as_ref().and_then(|frame| frame.element_with_id(element));

            // Render the element using the selection style
            element.map(|element| {
                properties.prepare_to_render(&mut drawing);
                element.render(&mut drawing, &properties);
            });

            // We'll draw the bounding rectangles later on
            bounding_boxes.push(bounds);
        }

        // Draw the bounding boxes
        drawing.new_path();
        drawing.extend(vec![
            Draw::NewPath
        ]);

        for bounds in bounding_boxes {
            bounds.draw(&mut drawing);
        }

        // Finish drawing the bounding boxes
        drawing.line_width_pixels(2.0);
        drawing.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 0.1));
        drawing.stroke();

        drawing.line_width_pixels(0.5);
        drawing.stroke_color(Color::Rgba(0.2, 0.8, 1.0, 1.0));
        drawing.stroke();

        // Finish up (popping state to restore the transformation)
        drawing.pop_state();
        drawing
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
    /// Returns true if the specified element ID is currently selected
    /// 
    fn is_selected(&self, data: &SelectData, item: ElementId) -> bool {
        data.selected_elements.contains(&item)
    }

    ///
    /// Returns the ID of the element at the position represented by the specified painting action
    /// 
    fn element_at_point(&self, data: &SelectData, point: (f32, f32)) -> Option<ElementId> {
        // Find the front-most item that matches this point
        for &(ref id, ref _props, ref bounding_box) in data.bounding_boxes.iter().rev() {
            if bounding_box.contains(point.0, point.1) {
                return Some(*id);
            }
        }

        // No ID matches
        None
    }

    ///
    /// Returns all of the elements that are contained within a particular area
    /// 
    fn elements_in_area(&self, data: &SelectData, point1: (f32, f32), point2: (f32, f32)) -> Vec<ElementId> {
        // Get the target rect
        let target = Rect::with_points(point1.0, point1.1, point2.0, point2.1).normalize();

        // Result is the IDs attached to the bounding boxes that overlap this rectangle
        data.bounding_boxes.iter()
            .filter(|&&(ref _id, ref _props, ref bounding_box)| bounding_box.overlaps(&target))
            .map(|&(ref id, ref _props, ref _bounding_box)| *id)
            .collect()
    }

    ///
    /// Processes a paint action (at the top level)
    /// 
    fn paint(&self, paint: Painting, actions: Vec<ToolAction<SelectData>>, data: Arc<SelectData>) -> (Vec<ToolAction<SelectData>>, Arc<SelectData>) {
        let mut actions     = actions;
        let mut data        = data;
        let current_action  = data.action;

        match (current_action, paint.action) {
            (_, PaintAction::Start) => {
                // Find the element at this point
                // TODO: preferentially check if the point is within the bounds of an already selected element
                let element = self.element_at_point(&*data, paint.location);

                if element.as_ref().map(|element| self.is_selected(&*data, *element)).unwrap_or(false) {
                    // Element is already selected: don't change the selection (so we can start dragging an existing selection)
                    let new_data = data.with_action(SelectAction::Reselect)
                        .with_initial_position(RawPoint::from(paint.location));

                    actions.push(ToolAction::Data(new_data.clone()));
                    data = Arc::new(new_data);

                } else if element.is_some() {
                    // This will select a new element (when the mouse is released)
                    let new_data = data.with_action(SelectAction::Select)
                        .with_initial_position(RawPoint::from(paint.location));

                    actions.push(ToolAction::Data(new_data.clone()));
                    data = Arc::new(new_data);

                } else {
                    // Clicking outside the current selection starts rubber-banding
                    let new_data = data.with_action(SelectAction::RubberBand)
                        .with_initial_position(RawPoint::from(paint.location));

                    actions.push(ToolAction::Data(new_data.clone()));
                    data = Arc::new(new_data);
                }
            },

            // -- Rubber-banding selection behaviour

            (SelectAction::Select, PaintAction::Continue) => {
                // TODO: only start rubber-banding once the mouse has moved a certain distance

                // Dragging after making a new selection moves us to rubber-band mode
                let mut new_data = data.with_action(SelectAction::RubberBand);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::Select, PaintAction::Finish) => {
                // Select whatever was at the initial position
                actions.push(ToolAction::ClearSelection);
                if let Some(selected) = self.element_at_point(&*data, data.initial_position.position) {
                    actions.push(ToolAction::Select(selected));
                }

                // Reset the action
                let mut new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::RubberBand, PaintAction::Continue) => {
                // Draw a rubber band around the selection
                let new_data = data.with_drag_position(RawPoint::from(paint.location));

                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                let draw_rubber_band = Self::draw_rubber_band(data.initial_position.position, paint.location);
                actions.push(ToolAction::Overlay(OverlayAction::Draw(draw_rubber_band)));
            },

            (SelectAction::RubberBand, PaintAction::Finish) => {
                // Reset the data state to 'no action'
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                // Select any items in this area
                actions.push(ToolAction::ClearSelection);
                data.drag_position.as_ref().map(|drag_position| {
                    actions.extend(self.elements_in_area(&data, data.initial_position.position, drag_position.position).into_iter().map(|item| ToolAction::Select(item)));
                });

                // Clear layer 1 (it's used to draw the rubber band)
                actions.push(ToolAction::Overlay(OverlayAction::Draw(vec![
                    Draw::Layer(1),
                    Draw::ClearLayer
                ])));
            },

            // -- Dragging behaviour

            (SelectAction::Reselect, PaintAction::Continue) => {
                // This begins a dragging operation
                let new_data = data.with_action(SelectAction::Drag);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::Drag, PaintAction::Continue) => {
                // Update the drag position
                let new_data = data.with_drag_position(RawPoint::from(paint.location));
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                // Draw the current drag state
                let selected_elements   = Arc::clone(&data.selected_elements);
                let selected            = data.bounding_boxes.iter()
                    .filter(|&&(ref id, _, _)| selected_elements.contains(id))
                    .map(|item| item.clone())
                    .collect();

                let draw_drag = Self::draw_drag(&*data, selected, data.initial_position.position, paint.location);
                actions.push(ToolAction::Overlay(OverlayAction::Draw(draw_drag)));
            },

            (SelectAction::Drag, PaintAction::Finish) => {
                // Reset the data state to 'no action'
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                // Redraw the selection highlights
                actions.push(ToolAction::Overlay(OverlayAction::Draw(vec![
                    Draw::Layer(1),
                    Draw::ClearLayer
                ])));
            },

            // -- Generic behaviour

            (_, PaintAction::Finish) => {
                // Reset the data state to 'no action'
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                // Clear layers other than 0 in the overlay
                actions.push(ToolAction::Overlay(OverlayAction::Draw(vec![
                    Draw::Layer(1),
                    Draw::ClearLayer
                ])));
            },

            (_, PaintAction::Cancel) => {
                // Reset the data state to 'no action'
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            // Other combinations have no effect
            _ => ()
        }

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
                let mut properties      = Arc::new(VectorProperties::default());
                let mut bounding_boxes  = vec![];

                for element in elements {
                    // Update the properties
                    properties = element.update_properties(properties);

                    // Get the paths for this element
                    let paths = element.to_path(&properties).unwrap_or(vec![]);

                    // Turn into a bounding box
                    let bounds = paths.into_iter()
                        .map(|path| path.bounding_box())
                        .fold(Rect::empty(), |current, next| current.union(next));

                    // Add to the result
                    bounding_boxes.push((element.id(), Arc::clone(&properties), bounds));
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
                    let mut properties  = Arc::new(VectorProperties::default());

                    for element in elements {
                        // Update the properties according to this element
                        properties = element.update_properties(properties);

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
        let current_frame       = tool_model.frame.clone();
        let bounding_boxes      = tool_model.bounding_boxes.clone();
        let selected_elements   = flo_model.selection().selected_element.clone();
        let data_for_model  = follow(computed(move || (current_frame.get(), selected_elements.get(), bounding_boxes.get())))
            .map(|(current_frame, selected_elements, bounding_boxes)| {
                ToolAction::Data(SelectData {
                    frame:              current_frame,
                    bounding_boxes:     bounding_boxes,
                    selected_elements:  Arc::new(selected_elements.into_iter().collect()),
                    action:             SelectAction::NoAction,
                    initial_position:   RawPoint::from((0.0, 0.0)),
                    drag_position:      None
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
