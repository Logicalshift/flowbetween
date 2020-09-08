use super::select_tool_model::*;
use crate::menu::*;
use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_canvas::*;
use flo_curves::*;
use flo_binding::*;
use flo_animation::*;

use futures::*;
use futures::stream;
use futures::stream::{BoxStream};
use std::sync::*;
use std::time::Duration;
use std::collections::{HashSet};

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
    Drag,

    /// The user is dragging one of the selection handles
    DragHandle(SelectHandle)
}

///
/// The handles that are used to manipulate the selection
///
#[derive(Copy, Clone, PartialEq, Debug)]
enum SelectHandle {
    ScaleTopLeft,
    ScaleTop,
    ScaleTopRight,
    ScaleRight,
    ScaleBottomRight,
    ScaleBottom,
    ScaleBottomLeft,
    ScaleLeft,

    Rotate
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

    /// The bounding box of the selected elements
    selection_bounds: Option<Rect>,

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
            selection_bounds:       self.selection_bounds.clone(),
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
            selection_bounds:       self.selection_bounds.clone(),
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
            selection_bounds:       self.selection_bounds.clone(),
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
    /// Returns the drawing instructions for the scaling handles for a particular bounding box
    ///
    fn scaling_handles_for_bounding_box(bounding_box: &Rect) -> Vec<Draw> {
        // Parameters for the handles
        let max_len         = 16.0;
        let separation      = 6.0;
        let gap             = 4.0;

        // The handles are placed on a bounding box outside the selection bounds
        let bounding_box    = bounding_box.inset(-separation, -separation);
        
        // Work out the length to draw the scaling handles
        let horiz_len       = if bounding_box.width() > (max_len*2.0 + gap) {
            max_len
        } else {
            ((bounding_box.width() - gap) / 2.0).floor()
        };
        let vert_len        = if bounding_box.height() > (max_len*2.0 + gap) {
            max_len
        } else {
            ((bounding_box.height() - gap) / 2.0).floor()
        };

        // Corner scaling handles
        let mut handles     = vec![];

        handles.extend(vec![
            Draw::NewPath,
            
            Draw::Move(bounding_box.x1, bounding_box.y1 + vert_len),
            Draw::Line(bounding_box.x1, bounding_box.y1),
            Draw::Line(bounding_box.x1 + horiz_len, bounding_box.y1),

            Draw::Move(bounding_box.x2 - horiz_len, bounding_box.y1),
            Draw::Line(bounding_box.x2, bounding_box.y1),
            Draw::Line(bounding_box.x2, bounding_box.y1 + vert_len),

            Draw::Move(bounding_box.x2, bounding_box.y2 - vert_len),
            Draw::Line(bounding_box.x2, bounding_box.y2),
            Draw::Line(bounding_box.x2 - horiz_len, bounding_box.y2),

            Draw::Move(bounding_box.x1 + horiz_len, bounding_box.y2),
            Draw::Line(bounding_box.x1, bounding_box.y2),
            Draw::Line(bounding_box.x1, bounding_box.y2 - vert_len)
        ]);

        // Edge scaling handles
        if (bounding_box.width() - horiz_len*2.0) > gap*2.0 + 2.0 {
            let mid_len     = bounding_box.width() - horiz_len*2.0 - gap * 2.0;
            let mid_len     = mid_len.min(max_len);
            let mid_point   = bounding_box.x1 + (bounding_box.width() - mid_len)/2.0;

            handles.extend(vec![
                Draw::Move(mid_point, bounding_box.y1),
                Draw::Line(mid_point+mid_len, bounding_box.y1),

                Draw::Move(mid_point, bounding_box.y2),
                Draw::Line(mid_point+mid_len, bounding_box.y2)
            ]);
        }

        if (bounding_box.height() - vert_len*2.0) > gap*2.0 + 2.0 {
            let mid_len     = bounding_box.height() - vert_len*2.0 - gap * 2.0;
            let mid_len     = mid_len.min(max_len);
            let mid_point   = bounding_box.y1 + (bounding_box.height() - mid_len)/2.0;

            handles.extend(vec![
                Draw::Move(bounding_box.x1, mid_point),
                Draw::Line(bounding_box.x1, mid_point+mid_len),

                Draw::Move(bounding_box.x2, mid_point),
                Draw::Line(bounding_box.x2, mid_point+mid_len)
            ]);
        }

        // Actually draw the handle lines
        handles.extend(vec!{
            Draw::LineWidthPixels(4.0),
            Draw::StrokeColor(SELECTION_OUTLINE),
            Draw::Stroke,

            Draw::LineWidthPixels(2.0),
            Draw::StrokeColor(SELECTION_BBOX),
            Draw::Stroke
        });

        handles
    }

    ///
    /// Returns drawing instructions for a rotation handle for the specified bounding box
    ///
    fn rotation_handle_for_bounding_box(bounding_box: &Rect) -> Vec<Draw> {
        // Properties
        let stalk_len   = 40.0;
        let radius      = 8.0;
        let separation  = 5.0;

        let mid_point   = bounding_box.x1 + bounding_box.width()/2.0;

        // Draw the stalk
        let mut handle  = vec![
            Draw::NewPath,
            Draw::Move(mid_point, bounding_box.y2+separation),
            Draw::Line(mid_point, bounding_box.y2+stalk_len-radius),

            Draw::LineWidthPixels(1.0),
            Draw::StrokeColor(SELECTION_OUTLINE),
            Draw::Stroke
        ];

        // Draw the 'rotator'
        handle.new_path();
        handle.circle(mid_point, bounding_box.y2+stalk_len, radius);
        handle.line_width_pixels(4.0);
        handle.stroke_color(SELECTION_OUTLINE);
        handle.stroke();
        handle.line_width_pixels(2.0);
        handle.stroke_color(SELECTION_BBOX);
        handle.stroke();

        handle
    }

    ///
    /// Returns how the specified selected elements should be rendered (as a selection)
    ///
    /// This can be used to cache the standard rendering for a set of selected elements so that we don't
    /// have to constantly recalculate it (which can be quite slow for a large set of brush elements)
    ///
    fn rendering_for_elements(data: &SelectData, selected_elements: &Vec<(ElementId, Arc<VectorProperties>, Rect)>) -> Vec<Draw> {
        let mut drawing = vec![];
        // let mut paths   = vec![];

        drawing.fill_color(Color::Rgba(0.6, 0.8, 0.9, 1.0));
        drawing.stroke_color(SELECTION_OUTLINE);
        drawing.new_path();

        // Draw each of the elements and store the bounding boxes
        for (element, properties, _bounds) in selected_elements.iter() {
            // Update the brush properties to be a 'shadow' of the original
            let mut properties = (**properties).clone();
            properties.brush_properties.opacity *= 0.25;
            properties.brush_properties.color   = Color::Rgba(0.6, 0.8, 0.9, 1.0);

            // Fetch the element to render it
            let element = data.frame.as_ref().and_then(|frame| frame.element_with_id(*element));

            // Render the element using the selection style
            element
                .and_then(|element| element.to_path(&properties, PathConversion::Fastest))
                .map(|element_paths| {
                    for path in element_paths {
                        drawing.extend(path.to_drawing());
                    }
                    /*
                    let path    = path_remove_interior_points::<_, _, Path>(&path, 0.01);

                    if properties.brush.to_definition().1 == BrushDrawingStyle::Erase {
                        paths   = path_sub::<_, _, _, Path>(&paths, &path, 0.01);
                    } else {
                        paths   = path_add::<_, _, _, Path>(&paths, &path, 0.01);
                    }
                    */
                });
        }

        // Draw the elements
        drawing.fill();
        drawing.stroke();

        // Draw the bounding boxes
        drawing.new_path();
        drawing.extend(vec![
            Draw::NewPath
        ]);

        // Return the set of commands for drawing these elements
        drawing
    }

    ///
    /// Renders the bounding box for a set of elements
    ///
    fn render_bounding_box(bounds: &Rect) -> Vec<Draw> {
        let mut drawing = vec![];

        bounds.draw(&mut drawing);

        // Finish drawing the bounding boxes
        drawing.line_width_pixels(2.0);
        drawing.stroke_color(SELECTION_OUTLINE);
        drawing.stroke();

        drawing.line_width_pixels(0.5);
        drawing.stroke_color(SELECTION_BBOX);
        drawing.stroke();

        // Draw the scaling handles
        drawing.extend(Self::scaling_handles_for_bounding_box(bounds));

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
        drawing.sprite_transform(SpriteTransform::Identity);

        // Draw the 'shadows' of the elements
        if data.selected_elements_draw.len() > 0 {
            // Use the cached version
            drawing.extend(&*data.selected_elements_draw);
        } else {
            // Regenerate every time
            drawing.extend(Self::rendering_for_elements(data, &selected_elements));
        }

        // Draw the bounding box
        let bounds = selected_elements.iter().map(|(_, _, bounds)| *bounds).fold(Rect::empty(), |r1, r2| r1.union(r2));
        drawing.extend(Self::render_bounding_box(&bounds));

        // Finish up (popping state to restore the transformation)
        drawing.pop_state();
        drawing
    }

    ///
    /// Returns the transformation to use during a drag of a selection handle
    ///
    fn handle_transformation(bounds: Rect, handle: SelectHandle, initial_point: (f32, f32), drag_point: (f32, f32)) -> (Coord2, ElementTransform) {
        let (drag_x, drag_y) = drag_point;
        let (init_x, init_y) = initial_point;

        // Origin depends on the handle being dragged (opposite side to the handle)
        let origin = match handle {
            SelectHandle::ScaleBottomRight  => (bounds.x1, bounds.y2),
            SelectHandle::ScaleBottom       => ((bounds.x1+bounds.x2)/2.0, bounds.y2),
            SelectHandle::ScaleBottomLeft   => (bounds.x2, bounds.y2),
            SelectHandle::ScaleLeft         => (bounds.x2, (bounds.y1+bounds.y2)/2.0),
            SelectHandle::ScaleTopLeft      => (bounds.x2, bounds.y1),
            SelectHandle::ScaleTop          => ((bounds.x1+bounds.x2)/2.0, bounds.y1),
            SelectHandle::ScaleTopRight     => (bounds.x1, bounds.y1),
            SelectHandle::ScaleRight        => (bounds.x1, (bounds.y1+bounds.y2)/2.0),
            SelectHandle::Rotate            => ((bounds.x1+bounds.x2)/2.0, (bounds.y1+bounds.y2)/2.0),
        };

        // The distance dragged in the x and y directions
        let dx = match handle {
            SelectHandle::ScaleTopLeft | SelectHandle::ScaleLeft | SelectHandle::ScaleBottomLeft        => init_x - drag_x,
            SelectHandle::ScaleTopRight | SelectHandle::ScaleRight | SelectHandle::ScaleBottomRight     => drag_x - init_x,
            SelectHandle::ScaleTop | SelectHandle::ScaleBottom                                          => 0.0,
            SelectHandle::Rotate                                                                        => {
                // dx will be the new position of the rotate handle relative to the origin
                let dx = drag_x - init_x;
                let px = (bounds.x1+bounds.x2)/2.0;

                px+dx-origin.0
            }
        };
        let dy = match handle {
            SelectHandle::ScaleTopLeft | SelectHandle::ScaleTop | SelectHandle::ScaleTopRight           => drag_y - init_y,
            SelectHandle::ScaleBottomLeft | SelectHandle::ScaleBottom | SelectHandle::ScaleBottomRight  => init_y - drag_y,
            SelectHandle::ScaleLeft | SelectHandle::ScaleRight                                          => 0.0,
            SelectHandle::Rotate                                                                        => {
                // dy will be the new position of the rotate handle relative to the origin
                let dy = drag_y - init_y;
                let py = bounds.y2+40.0;

                py+dy-origin.1
            }
        };

        // TODO: for even scaling make dx, dy equal to the max of both

        match handle {
            SelectHandle::Rotate => {
                // Rotate the handle around the center
                let theta       = f32::atan2(-dx, dy);

                let (ox, oy)    = origin;
                (Coord2(ox as f64, oy as f64), ElementTransform::Rotate(theta as f64))
            }

            _ => {
                // For scaling, work out a new bounding box size (the differences between the scaling algorithm are all specified by the origin and the dx and dy values)
                let scale_x     = (bounds.width() + dx) / bounds.width();
                let scale_y     = (bounds.height() + dy) / bounds.height();

                let (ox, oy)    = origin;
                (Coord2(ox as f64, oy as f64), ElementTransform::Scale(scale_x as f64, scale_y as f64))
            }
        }
    }

    ///
    /// Draws a set of dragged elements
    ///
    fn draw_drag_handle(data: &SelectData, handle: SelectHandle, selected_elements: Vec<(ElementId, Arc<VectorProperties>, Rect)>, initial_point: (f32, f32), drag_point: (f32, f32)) -> Vec<Draw> {
        let mut drawing = vec![];

        drawing.layer(1);
        drawing.clear_layer();

        // Work out the bounding box
        let bounds = selected_elements.iter()
            .map(|(_, _, bounds)| *bounds)
            .fold(Rect::empty(), |r1, r2| r1.union(r2));

        // Convert to a transformation
        let (origin, transform) = Self::handle_transformation(bounds, handle, initial_point, drag_point);
        let transform           = Transformation::from_element_transform(&origin, transform);

        // Draw everything translated by the drag distance
        drawing.push_state();
        drawing.transform(transform.clone().into());

        // Draw the 'shadows' of the elements
        if data.selected_elements_draw.len() > 0 {
            // Use the cached version
            drawing.extend(&*data.selected_elements_draw);
        } else {
            // Regenerate every time
            drawing.extend(Self::rendering_for_elements(data, &selected_elements));
        }

        // Draw the bounding box
        match transform {
            Transformation::Scale(_, _, _) => { 
                // Reset to the default state (so we can draw the bounding box untransformed - it'll look weird with a 2D transform applied to it)
                drawing.pop_state();
                drawing.push_state();

                // Transform the bounds
                let Coord2(x1, y1)  = transform.transform_point(&Coord2(bounds.x1 as f64, bounds.y1 as f64));
                let Coord2(x2, y2)  = transform.transform_point(&Coord2(bounds.x2 as f64, bounds.y2 as f64));

                // Render the transformed bounding box
                let bounds          = Rect { x1: x1 as f32, y1: y1 as f32, x2: x2 as f32, y2: y2 as f32 };
                drawing.extend(Self::render_bounding_box(&bounds));
            }

            _ => { 
                drawing.extend(Self::render_bounding_box(&bounds));
                drawing.extend(Self::rotation_handle_for_bounding_box(&bounds));
            }
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
        let paths = element.to_path(properties, PathConversion::Fastest);
        if let Some(paths) = paths {
            let bounds = paths.iter()
                .map(|path| path.bounding_box())
                .fold(Rect::empty(), |r1, r2| r1.union(r2));

            let mut path_draw: Vec<_> = paths.into_iter()
                .flat_map(|path| { let path: Vec<Draw> = (&path).into(); path })
                .collect();

            path_draw.insert(0, Draw::NewPath);

            // Draw the outline
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
    fn element_at_point<IsSelected: Fn(ElementId) -> bool>(model: &FrameModel, element_is_selected: IsSelected, point: (f32, f32)) -> Option<ElementId> {
        // Find all of the elements at this point
        let elements = model.elements_at_point(point);

        // Choose an element inside the path if possible, otherwise pick an already selected elements or an element in bounds
        let mut fallback_selection = None;

        for element_match in elements {
            match element_match {
                ElementMatch::OnlyInBounds(element_id) => {
                    // Use as a fallback
                    if fallback_selection.is_none() { fallback_selection = Some(element_id) }

                    // Already selected elements take precedence over previous fallbacks
                    if element_is_selected(element_id) { fallback_selection = Some(element_id) }
                },

                ElementMatch::InsidePath(element_id) => {
                    // Choose this element
                    return Some(element_id);
                }
            }
        }

        // No elements in path, so the result is the fallback selection
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
    /// Returns the selection handle found at the specified point
    ///
    fn handle_at_point(bounds: Rect, point: (f32, f32)) -> Option<SelectHandle> {
        // Parameters for the handles
        let max_len             = 16.0;
        let separation          = 6.0;
        let gap                 = 4.0;
        let stalk_len           = 40.0;
        let radius              = 8.0;

        // The handles are placed on a bounding box outside the selection bounds
        let bounding_box        = bounds.inset(-separation, -separation);
        
        // Work out the length to draw the scaling handles
        let horiz_len           = if bounding_box.width() > (max_len*2.0 + gap) {
            max_len
        } else {
            ((bounding_box.width() - gap) / 2.0).floor()
        };
        let vert_len            = if bounding_box.height() > (max_len*2.0 + gap) {
            max_len
        } else {
            ((bounding_box.height() - gap) / 2.0).floor()
        };

        // Compute which handle the pointer is over
        let (x, y)              = point;
        let (x1, y1, x2, y2)    = (bounds.x1, bounds.y1, bounds.x2, bounds.y2);
        let (mid_x, mid_y)      = ((bounds.x1+bounds.x2)/2.0, (bounds.y1+bounds.y2)/2.0);

        // Scale handles
        let border = 1.0;

        if x <= x1+border && x >= x1-separation {
            if y >= y2-vert_len && y <= y2 + separation {
                return Some(SelectHandle::ScaleTopLeft)
            } else if y >= y1 - separation && y <= y1+vert_len {
                return Some(SelectHandle::ScaleBottomLeft)
            } else if y >= mid_y - vert_len/2.0 && y <= mid_y + vert_len/2.0 {
                return Some(SelectHandle::ScaleLeft)
            }
        }

        if x >= x2-border && x <= x2+separation {
            if y >= y2-vert_len && y <= y2 + separation {
                return Some(SelectHandle::ScaleTopRight)
            } else if y >= y1 - separation && y <= y1+vert_len {
                return Some(SelectHandle::ScaleBottomRight)
            } else if y >= mid_y - vert_len/2.0 && y <= mid_y + vert_len/2.0 {
                return Some(SelectHandle::ScaleRight)
            }
        }

        if y <= y1+border && y >= y1-separation {
            if x >= x1-separation && x <= x1+horiz_len {
                return Some(SelectHandle::ScaleBottomLeft)
            } else if x >= x2-horiz_len && x <= x2+separation {
                return Some(SelectHandle::ScaleBottomRight)
            } else if x >= mid_x - horiz_len/2.0 && x <= mid_x + horiz_len/2.0 {
                return Some(SelectHandle::ScaleBottom)
            }
        }

        if y >= y2-border && y <= y2+separation {
            if x >= x1-separation && x <= x1+horiz_len {
                return Some(SelectHandle::ScaleTopLeft)
            } else if x >= x2-horiz_len && x <= x2+separation {
                return Some(SelectHandle::ScaleTopRight)
            } else if x >= mid_x - horiz_len/2.0 && x <= mid_x + horiz_len/2.0 {
                return Some(SelectHandle::ScaleTop)
            }
        }

        if y >= y2+stalk_len-radius-separation && y <= y2+stalk_len+radius
            && x >= mid_x-radius && x <= mid_x+radius {
            return Some(SelectHandle::Rotate)
        }

        // Over no handles
        None
    }

    ///
    /// Processes a paint action (at the top level)
    ///
    fn paint<Anim: 'static+EditableAnimation>(&self, paint: Painting, actions: Vec<ToolAction<SelectData>>, animation: Arc<FloModel<Anim>>, data: Arc<SelectData>) -> (Vec<ToolAction<SelectData>>, Arc<SelectData>) {
        let mut actions     = actions;
        let mut data        = data;
        let current_action  = data.action;
        let select_bounds   = data.selection_bounds.clone();

        match (current_action, paint.action) {
            (_, PaintAction::Start) => {
                // Find the element at this point
                // TODO: preferentially check if the point is within the bounds of an already selected element
                let element = Self::element_at_point(&*animation.frame(), |element_id| self.is_selected(&data, element_id), paint.location);
                let handle  = select_bounds.and_then(|bounds| Self::handle_at_point(bounds, paint.location));

                if let Some(handle) = handle {
                    // User has clicked over a handle (these drag for various effects)
                    let new_data = data.with_action(SelectAction::DragHandle(handle))
                        .with_initial_position(RawPoint::from(paint.location));

                    actions.push(ToolAction::Data(new_data.clone()));
                    data = Arc::new(new_data);

                } else if element.as_ref().map(|element| self.is_selected(&*data, *element)).unwrap_or(false) {
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

            (SelectAction::Select, PaintAction::Continue)   |
            (SelectAction::Select, PaintAction::Prediction) => {
                // TODO: only start rubber-banding once the mouse has moved a certain distance

                // Dragging after making a new selection moves us to rubber-band mode
                let new_data = data.with_action(SelectAction::RubberBand);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::Select, PaintAction::Finish) => {
                // Select whatever was at the initial position
                actions.push(ToolAction::ClearSelection);
                if let Some(selected) = Self::element_at_point(&*animation.frame(), |element_id| self.is_selected(&data, element_id), data.initial_position.position) {
                    actions.push(ToolAction::Select(selected));
                }

                // Reset the action
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::RubberBand, PaintAction::Continue)   |
            (SelectAction::RubberBand, PaintAction::Prediction) => {
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

            (SelectAction::Reselect, PaintAction::Continue)   |
            (SelectAction::Reselect, PaintAction::Prediction) => {
                // This begins a dragging operation
                let mut new_data        = data.with_action(SelectAction::Drag);

                // Pre-render the elements so we can draw the drag faster
                let selected_elements   = Arc::clone(&data.selected_elements);
                let selected            = data.bounding_boxes.iter()
                    .filter(|&&(ref id, _, _)| selected_elements.contains(id))
                    .map(|item| item.clone())
                    .collect();
                let render              = Self::rendering_for_elements(&new_data, &selected);

                // Define the rendering as the selection sprite
                actions.push(ToolAction::Overlay(OverlayAction::Draw(
                    vec![Draw::Sprite(SPRITE_SELECTION_OUTLINE)].into_iter()
                        .chain(render.into_iter())
                        .chain(vec![Draw::Layer(0)])
                        .collect()

                )));

                new_data.selected_elements_draw = Arc::new(vec![Draw::DrawSprite(SPRITE_SELECTION_OUTLINE)]);

                // Update the tool data
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);
            },

            (SelectAction::Drag, PaintAction::Continue)   |
            (SelectAction::Drag, PaintAction::Prediction) => {
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

            (SelectAction::DragHandle(handle), PaintAction::Continue)   |
            (SelectAction::DragHandle(handle), PaintAction::Prediction) => {
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

                let draw_drag = Self::draw_drag_handle(&*data, handle, selected, data.initial_position.position, paint.location);
                actions.push(ToolAction::Overlay(OverlayAction::Draw(draw_drag)));
            },

            (SelectAction::DragHandle(handle), PaintAction::Finish) => {
                // Reset the data state to 'no action'
                let new_data = data.with_action(SelectAction::NoAction);
                actions.push(ToolAction::Data(new_data.clone()));
                data = Arc::new(new_data);

                // Transform these elements
                // (TODO: also support motions for these kinds of transformations, and support raw transformations for the elements themselves)
                let selected_element_ids    = data.selected_elements.iter().cloned().collect();
                let (origin, transform)     = Self::handle_transformation(data.selection_bounds.unwrap_or(Rect::empty()), handle, data.initial_position.position, paint.location);
                let transform_elements      = vec![ElementTransform::SetAnchor(origin.x(), origin.y()), transform];
                let transform_elements      = vec![AnimationEdit::Element(selected_element_ids, ElementEdit::Transform(transform_elements))];

                actions.extend(transform_elements.into_iter().map(|elem| ToolAction::Edit(elem)));

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

impl<Anim: 'static+EditableAnimation+Animation> Tool<Anim> for Select {
    type ToolData   = SelectData;
    type Model      = SelectToolModel;

    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    ///
    /// Creates the model for the Select tool
    ///
    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> SelectToolModel {
        SelectToolModel::new(flo_model.selection())
    }

    ///
    /// Creates the menu bar controller for the select tool
    ///
    fn create_menu_controller(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &SelectToolModel) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(SelectMenuController::new(&*flo_model, tool_model)))
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    ///
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, _tool_model: &SelectToolModel) -> BoxStream<'static, ToolAction<SelectData>> {
        // The set of currently selected elements
        let selected_elements   = flo_model.selection().selected_elements.clone();

        // Create a binding that works out the frame for the currently selected layer
        let current_frame       = flo_model.frame().frame.clone();

        // Follow it, and draw an overlay showing the bounding boxes of everything that's selected
        let draw_selection_overlay = follow(computed(move || (current_frame.get(), selected_elements.get())))
            .map(|(current_frame, selected_elements)| {
                if let Some(current_frame) = current_frame {
                    // Build up a vector of bounds
                    let mut selection   = vec![];
                    let mut bounds      = Rect::empty();

                    // Draw highlights around the selection (and discover the bounds)
                    for selected_id in selected_elements.iter() {
                        let element = current_frame.element_with_id(*selected_id);

                        if let Some(element) = element {
                            // Update the properties according to this element
                            let properties  = current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

                            // Draw a highlight around it
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

                        // Draw the scaling handles (TODO: except when the user is dragging the selection)
                        selection.extend(Self::scaling_handles_for_bounding_box(&bounds));
                        selection.extend(Self::rotation_handle_for_bounding_box(&bounds));
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

        // Combine the elements and the bounding boxes into a single vector
        let elements                = flo_model.frame().elements.clone();
        let bounding_boxes          = flo_model.frame().bounding_boxes.clone();
        let combined_bounding_boxes = computed(move || {
            let elements            = elements.get();
            let bounding_boxes      = bounding_boxes.get();

            Arc::new(elements.iter().map(|(element, properties)| {
                let properties  = Arc::clone(properties);
                let element_id  = element.id();
                let bounds      = bounding_boxes.get(&element_id).cloned().unwrap_or_else(|| Rect::empty());

                (element_id, properties, bounds)
            }).collect::<Vec<_>>())
        });

        // Whenever the frame or the set of bounding boxes changes, we create a new SelectData object
        // (this also resets any in-progress action)
        let current_frame       = flo_model.frame().frame.clone();
        let selected_elements   = flo_model.selection().selected_elements.clone();
        let data_for_model  = follow(computed(move || (current_frame.get(), selected_elements.get(), combined_bounding_boxes.get())))
            .map(|(current_frame, selected_elements, combined_bounding_boxes)| {
                // Collapse the bounding boxes to the selection bounds
                let selection_bounds = (*combined_bounding_boxes).iter()
                    .fold(None, |maybe_bounds: Option<Rect>, (element_id, _, next_rect)| {
                        if selected_elements.contains(element_id) {
                            match maybe_bounds {
                                Some(bounds)    => Some(bounds.union(*next_rect)),
                                None            => Some(*next_rect)
                            }
                        } else {
                            maybe_bounds
                        }
                    });

                ToolAction::Data(SelectData {
                    frame:                  current_frame,
                    bounding_boxes:         combined_bounding_boxes,
                    selected_elements:      selected_elements.clone(),
                    selected_elements_draw: Arc::new(vec![]),
                    selection_bounds:       selection_bounds,
                    action:                 SelectAction::NoAction,
                    initial_position:       RawPoint::from((0.0, 0.0)),
                    drag_position:          None
                })
            });

        // Generate the final stream
        let select_stream = stream::select(data_for_model, draw_selection_overlay);
        Box::pin(select_stream)
    }

    ///
    /// Returns the actions that result from a particular inpiut
    ///
    fn actions_for_input<'a>(&self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<SelectData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<SelectData>>>) -> Box<dyn Iterator<Item=ToolAction<SelectData>>> {
        if let Some(mut data) = data {
            // We build up a vector of actions to perform as we go
            let mut actions = vec![];
            let input       = ToolInput::last_paint_actions_only(input);

            // Process the inputs (reversing the filter again so they're back in order)
            for input in input {
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
