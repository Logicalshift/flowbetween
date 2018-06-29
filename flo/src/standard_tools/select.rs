use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;
use super::super::style::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;
use curves::bezier::path::path_contains_point;

use futures::*;
use std::sync::*;
use std::time::Duration;
use std::collections::HashSet;

///
/// The model for the Select tool
/// 
#[derive(Clone)]
pub struct SelectModel {
    /// Contains a pointer to the current frame
    frame: BindRef<Option<Arc<dyn Frame>>>,

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
    frame: Option<Arc<dyn Frame>>,

    // The bounding boxes of the elements in the current frame
    bounding_boxes: Arc<Vec<(ElementId, Arc<VectorProperties>, Rect)>>,

    // The current set of selected elements
    selected_elements: Arc<HashSet<ElementId>>,

    // The drawing instructions to render the selected elements (or empty if there's no rendering yet)
    selected_elements_draw: Arc<Vec<Draw>>,

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
            frame:                  self.frame.clone(),
            bounding_boxes:         self.bounding_boxes.clone(),
            selected_elements:      self.selected_elements.clone(),
            selected_elements_draw: self.selected_elements_draw.clone(),
            action:                 new_action,
            initial_position:       self.initial_position.clone(),
            drag_position:          self.drag_position.clone()
        }
    }
    
    ///
    /// Creates a copy of this object with a new initial position
    ///
    fn with_initial_position(&self, new_initial_position: RawPoint) -> SelectData {
        SelectData {
            frame:                  self.frame.clone(),
            bounding_boxes:         self.bounding_boxes.clone(),
            selected_elements:      self.selected_elements.clone(),
            selected_elements_draw: self.selected_elements_draw.clone(),
            action:                 self.action,
            initial_position:       new_initial_position,
            drag_position:          None
        }
    }

    ///
    /// Creates a copy of this object with a new drag position
    /// 
    fn with_drag_position(&self, new_drag_position: RawPoint) -> SelectData {
        SelectData {
            frame:                  self.frame.clone(),
            bounding_boxes:         self.bounding_boxes.clone(),
            selected_elements:      self.selected_elements.clone(),
            selected_elements_draw: self.selected_elements_draw.clone(),
            action:                 self.action,
            initial_position:       self.initial_position.clone(),
            drag_position:          Some(new_drag_position)
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
            Draw::StrokeColor(SELECTION_BBOX),
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
            Draw::StrokeColor(RUBBERBAND_OUTLINE),
            Draw::Stroke
        ];

        let draw_inner = vec![
            Draw::LineWidthPixels(0.5),
            Draw::StrokeColor(RUBBERBAND_LINE),
            Draw::Stroke
        ];

        let draw_fill = vec![
            Draw::FillColor(RUBBERBAND_FILL),
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
    /// Returns how the specified selected elements should be rendered (as a selection)
    /// 
    /// This can be used to cache the standard rendering for a set of selected elements so that we don't
    /// have to constantly recalculate it (which can be quite slow for a large set of brush elements)
    /// 
    fn rendering_for_elements(data: &SelectData, selected_elements: Vec<(ElementId, Arc<VectorProperties>, Rect)>) -> Vec<Draw> {
        let mut drawing = vec![];

        // Draw each of the elements and store the bounding boxes
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

        let bounds = bounding_boxes.into_iter().fold(Rect::empty(), |r1, r2| r1.union(r2));
        bounds.draw(&mut drawing);

        // Finish drawing the bounding boxes
        drawing.line_width_pixels(2.0);
        drawing.stroke_color(SELECTION_OUTLINE);
        drawing.stroke();

        drawing.line_width_pixels(0.5);
        drawing.stroke_color(SELECTION_BBOX);
        drawing.stroke();

        // Return the set of commands for drawing these elements
        drawing
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
        if data.selected_elements_draw.len() > 0 {
            // Use the cached version
            drawing.extend(&*data.selected_elements_draw);
        } else {
            // Regenerate every time
            drawing.extend(Self::rendering_for_elements(data, selected_elements));
        }

        // Finish up (popping state to restore the transformation)
        drawing.pop_state();
        drawing
    }

    ///
    /// Returns the drawing actions to highlight the specified element
    /// 
    fn highlight_for_selection(element: &Vector, properties: &VectorProperties) -> (Vec<Draw>, Rect) {
        // Get the paths for this element
        let paths = element.to_path(properties);
        if let Some(paths) = paths {
            let bounds = paths.iter()
                .map(|path| path.bounding_box())
                .fold(Rect::empty(), |r1, r2| r1.union(r2));

            let mut path_draw: Vec<_> = paths.into_iter()
                .flat_map(|path| { let path: Vec<Draw> = (&path).into(); path })
                .collect();
            
            path_draw.insert(0, Draw::NewPath);

            path_draw.push(Draw::FillColor(SELECTION_FILL));
            path_draw.push(Draw::Fill);

            path_draw.push(Draw::StrokeColor(SELECTION_OUTLINE));
            path_draw.push(Draw::LineWidthPixels(2.0));
            path_draw.push(Draw::Stroke);

            path_draw.push(Draw::StrokeColor(SELECTION_HIGHLIGHT));
            path_draw.push(Draw::LineWidthPixels(0.5));
            path_draw.push(Draw::Stroke);

            (path_draw, bounds)
        } else {
            // There are no paths for this element
            (vec![], Rect::empty())
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
    fn element_at_point(data: &SelectData, point: (f32, f32)) -> Option<ElementId> {
        let mut fallback_selection = None;

        // Find the front-most item that matches this point
        for &(ref id, ref props, ref bounding_box) in data.bounding_boxes.iter().rev() {
            if bounding_box.contains(point.0, point.1) {
                // Use this as the fallback if there isn't one already
                if fallback_selection.is_none() { fallback_selection = Some(*id); }

                // Already selected elements take precedence when picking a fallback
                if data.selected_elements.contains(id) { fallback_selection = Some(*id); }

                // Get the paths for this element
                let paths = data.frame
                    .as_ref()
                    .and_then(|frame| frame.element_with_id(*id))
                    .and_then(|element| element.to_path(props));

                // If there are any paths, then see if the point is inside them
                // TODO: path_contains_point won't work if the paths contain any move elements apart from the initial one
                if let Some(paths) = paths {
                    if paths.into_iter().any(|path| path_contains_point(&path, &PathPoint::new(point.0, point.1))) {
                        return Some(*id)
                    }
                }
            }
        }

        // No ID matches precisely (but we may have found a fallback match based on the bounding box)
        fallback_selection
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
    fn paint<Anim: 'static+Animation>(&self, paint: Painting, actions: Vec<ToolAction<SelectData>>, animation: Arc<FloModel<Anim>>, data: Arc<SelectData>) -> (Vec<ToolAction<SelectData>>, Arc<SelectData>) {
        let mut actions     = actions;
        let mut data        = data;
        let current_action  = data.action;

        match (current_action, paint.action) {
            (_, PaintAction::Start) => {
                // Find the element at this point
                // TODO: preferentially check if the point is within the bounds of an already selected element
                let element = Self::element_at_point(&*data, paint.location);

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
                if let Some(selected) = Self::element_at_point(&*data, data.initial_position.position) {
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
                let mut new_data = data.with_action(SelectAction::Drag);

                // Pre-render the elements so we can draw the drag faster
                let selected_elements   = Arc::clone(&data.selected_elements);
                let selected            = data.bounding_boxes.iter()
                    .filter(|&&(ref id, _, _)| selected_elements.contains(id))
                    .map(|item| item.clone())
                    .collect();
                new_data.selected_elements_draw = Arc::new(Self::rendering_for_elements(&new_data, selected));

                // Update the tool data
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

                // Create a motion for this element
                let selected_element_ids    = data.selected_elements.iter().cloned().collect();
                let edit_time               = data.frame.as_ref().map(|frame| frame.time_index()).unwrap_or(Duration::from_millis(0));
                let move_elements           = MotionEditAction::MoveElements(selected_element_ids, edit_time, data.initial_position.position, paint.location);

                actions.extend(move_elements.to_animation_edits(&*animation).into_iter().map(|elem| ToolAction::Edit(elem)));

                // Cause the frame to be redrawn
                actions.push(ToolAction::InvalidateFrame);

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
        let current_frame   = flo_model.frame().frame.clone();

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
    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &SelectModel) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(SelectMenuController::new()))
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    /// 
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &SelectModel) -> Box<dyn Stream<Item=ToolAction<SelectData>, Error=()>+Send> {
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
                    let mut bounds      = Rect::empty();

                    for element in elements {
                        // Update the properties according to this element
                        properties = element.update_properties(properties);

                        // If the element is selected, draw a highlight around it
                        let element_id = element.id();
                        if element_id.is_assigned() && selected_elements.contains(&element_id) {
                            // Draw the settings for this element
                            let (drawing, bounding_box) = Self::highlight_for_selection(&element, &properties);
                            selection.extend(drawing);
                            bounds = bounds.union(bounding_box);
                        }
                    }

                    // Draw a bounding box around the whole thing
                    if !bounds.is_zero_size() {
                        let bounds = bounds.inset(-2.0, -2.0);

                        bounds.draw(&mut selection);

                        selection.line_width_pixels(2.0);
                        selection.stroke_color(SELECTION_OUTLINE);
                        selection.stroke();

                        selection.line_width_pixels(0.5);
                        selection.stroke_color(SELECTION_BBOX);
                        selection.stroke();
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
                    frame:                  current_frame,
                    bounding_boxes:         bounding_boxes,
                    selected_elements:      Arc::new(selected_elements.into_iter().collect()),
                    selected_elements_draw: Arc::new(vec![]),
                    action:                 SelectAction::NoAction,
                    initial_position:       RawPoint::from((0.0, 0.0)),
                    drag_position:          None
                })
            });
        
        // Generate the final stream
        let select_stream = data_for_model.select(draw_selection_overlay);
        Box::new(select_stream)
    }

    ///
    /// Returns the actions that result from a particular inpiut
    /// 
    fn actions_for_input<'a>(&self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<SelectData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<SelectData>>>) -> Box<dyn Iterator<Item=ToolAction<SelectData>>> {
        if let Some(mut data) = data {
            // We build up a vector of actions to perform as we go
            let mut actions = vec![];

            // Filter the input so that there is only a single paint continue event
            let input: Vec<_>               = input.collect();
            let mut seen_continue           = false;
            let mut reversed_filtered_input = vec![];

            // Reverse the inputs so the first continue we see is the most recent
            for input in input.into_iter().rev() {
                match input {
                    ToolInput::Paint(painting) => {
                        if painting.action == PaintAction::Continue {
                            if !seen_continue {
                                // Only push the first continue
                                reversed_filtered_input.push(ToolInput::Paint(painting));
                                seen_continue = true;
                            }
                        } else {
                            // All other painting actions make it through
                            reversed_filtered_input.push(ToolInput::Paint(painting));
                        }
                    },

                    input => reversed_filtered_input.push(input)
                }
            }

            // Process the inputs (reversing the filter again so they're back in order)
            for input in reversed_filtered_input.into_iter().rev() {
                match input {
                    ToolInput::Data(new_data) => {
                        // Whenever we get feedback about what the data is set to, update our data
                        data = new_data;
                    },

                    ToolInput::Select | ToolInput::Deselect => {
                        // Reset the action to 'no action' when the tool is selected or deselected
                        let mut new_data = data.with_action(SelectAction::NoAction);

                        // Clear the element rendering
                        new_data.selected_elements_draw = Arc::new(vec![]);

                        // This replaces the data object
                        data = Arc::new(new_data.clone());

                        // And we get an action to update the data for the next set of inputs
                        actions.push(ToolAction::Data(new_data));
                    },

                    ToolInput::Paint(painting)  => {
                        let (new_actions, new_data) = self.paint(painting, actions, flo_model.clone(), data);
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
