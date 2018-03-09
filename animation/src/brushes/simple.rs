use super::super::traits::*;

use curves::*;
use curves::bezier;
use canvas::*;

use std::iter;

///
/// Simple brush, which renders a brush stroke as a straight series of line segments
/// 
pub struct SimpleBrush {

}

impl SimpleBrush {
    pub fn new() -> SimpleBrush {
        SimpleBrush { }
    }
}

impl Brush for SimpleBrush {
    fn prepare_to_render<'a>(&'a self, properties: &BrushProperties) -> Box<'a+Iterator<Item=Draw>> {
        Box::new(vec![
            Draw::BlendMode(BlendMode::SourceOver),
            Draw::StrokeColor(properties.color)
        ].into_iter())
    }

    ///
    /// Retrieves the definition for this brush
    /// 
    fn to_definition(&self) -> (BrushDefinition, BrushDrawingStyle) {
        (BrushDefinition::Simple, BrushDrawingStyle::Draw)
    }

    ///
    /// Returns the brush points for rendering given a particular set of raw points
    /// 
    fn brush_points_for_raw_points(&self, points: &[RawPoint]) -> Vec<BrushPoint> {
        // Map to coordinates
        let coords: Vec<_> = points.iter().map(|point| Coord2(point.position.0 as f64, point.position.1 as f64)).collect();

        // Pick points that are at least a certain distance apart to use for the fitting algorithm
        let mut distant_coords  = vec![];
        let mut last_point      = coords[0];

        distant_coords.push(last_point);
        for x in 1..coords.len() {
            if last_point.distance_to(&coords[x]) >= 2.0 {
                last_point = coords[x];
                distant_coords.push(last_point);
            }
        }

        // Fit these points to a curve
        let curve = bezier::Curve::fit_from_points(&distant_coords, 2.0);

        // Turn into brush points
        let mut brush_points = vec![];

        if let Some(curve) = curve {
            // First point is the start point, the control points don't matter for this
            let start = curve[0].start_point();;
            brush_points.push(BrushPoint {
                position:   (start.x() as f32, start.y() as f32),
                cp1:        (0.0, 0.0),
                cp2:        (0.0, 0.0),
                width:      0.0
            });

            // Convert the remaining curve segments
            for segment in curve {
                let end             = segment.end_point();
                let (cp1, cp2)      = segment.control_points();

                brush_points.push(BrushPoint {
                    position:   (end.x() as f32, end.y() as f32),
                    cp1:        (cp1.x() as f32, cp1.y() as f32),
                    cp2:        (cp2.x() as f32, cp2.y() as f32),
                    width:      1.0
                });
            }
        }

        brush_points
    }

    fn render_brush<'a>(&'a self, _properties: &'a BrushProperties, points: &'a Vec<BrushPoint>) -> Box<'a+Iterator<Item=Draw>> {
        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 1 {
            return Box::new(iter::empty());
        }
        
        // Draw a simple line for this brush
        let preamble = vec![
            Draw::NewPath,
            Draw::Move(points[0].position.0, points[0].position.1)
        ];

        let curves = points.iter()
            .skip(1)
            .map(|segment| Draw::BezierCurve(
                (segment.position.0, segment.position.1),
                (segment.cp1.0, segment.cp1.1),
                (segment.cp2.0, segment.cp2.1)
            ));

        Box::new(preamble.into_iter()
            .chain(curves)
            .chain(iter::once(Draw::Stroke)))
    }
}