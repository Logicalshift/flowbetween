use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;
use super::super::style::*;

use ui::*;
use canvas::*;
use animation::*;

use itertools::*;
use std::sync::*;

///
/// The Adjust tool (adjusts control points of existing objects)
/// 
pub struct Adjust { }

impl Adjust {
    ///
    /// Creates a new instance of the Adjust tool
    /// 
    pub fn new() -> Adjust {
        Adjust {}
    }

    ///
    /// Draws an element control point
    /// 
    pub fn draw_control_point(cp: &ControlPoint) -> Vec<Draw> {
        let mut draw = vec![];

        draw.new_path();

        match cp {
            ControlPoint::BezierPoint(x, y) => {
                draw.circle(*x, *y, 6.0);
                draw.fill_color(CP_BEZIER);
                draw.fill();
                draw.stroke();
            },

            ControlPoint::BezierControlPoint(x, y) => {
                draw.rect(x-4.0, y-4.0, x+4.0, y+4.0);
                draw.fill_color(CP_BEZIER_CP);
                draw.fill();
                draw.stroke();
            }
        };

        draw
    }

    ///
    /// Returns the drawing instructions for the control points for an element
    /// 
    pub fn control_points_for_element<Elem: VectorElement>(element: &Elem, properties: &VectorProperties) -> Vec<Draw> {
        // Create the vector where the control points will be drawn
        let mut draw = vec![];

        // Outline the path
        let paths = element.to_path(properties);

        if let Some(paths) = paths {
            draw.new_path();
            draw.extend(paths.into_iter().flat_map(|path| -> Vec<Draw> { (&path).into() }));

            draw.stroke_color(SELECTION_OUTLINE);
            draw.line_width_pixels(2.0);
            draw.stroke();

            draw.stroke_color(SELECTION_HIGHLIGHT);
            draw.line_width_pixels(0.5);
            draw.stroke();
        }

        let control_points = element.control_points();

        // Draw the control point connecting lines
        draw.new_path();
        for (prev, next) in control_points.iter().tuple_windows() {
            match (prev, next) {
                (ControlPoint::BezierPoint(x1, y1), ControlPoint::BezierControlPoint(x2, y2)) |
                (ControlPoint::BezierControlPoint(x2, y2), ControlPoint::BezierPoint(x1, y1)) => {
                    draw.move_to(*x1, *y1);
                    draw.line_to(*x2, *y2);
                },

                _ => ()
            }
        }

        draw.line_width_pixels(2.0);
        draw.stroke_color(SELECTION_OUTLINE);
        draw.stroke();
        draw.line_width_pixels(1.0);
        draw.stroke_color(CP_LINES);
        draw.stroke();

        // Draw the control points themselves
        draw.stroke_color(SELECTION_OUTLINE);
        draw.line_width_pixels(1.0);

        for cp in control_points.iter() {
            draw.extend(Self::draw_control_point(cp));
        }

        draw
    }
}

impl<Anim: Animation> Tool<Anim> for Adjust {
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(AdjustMenuController::new()))
    }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<()>>, _input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
