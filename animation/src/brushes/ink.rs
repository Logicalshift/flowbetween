use super::super::traits::*;
use ui::canvas::*;

use std::ops::*;

use curves::*;
use curves::bezier;

///
/// The ink brush draws a solid line with width based on pressure
/// 
pub struct InkBrush {
    /// Width at pressure 0%
    min_width: f32,

    /// Width at pressure 100%
    max_width: f32,
}

impl InkBrush {
    ///
    /// Creates a new ink brush with the default settings
    /// 
    pub fn new() -> InkBrush {
        InkBrush { 
            min_width: 2.0,
            max_width: 10.0
        }
    }
}

///
/// Ink brush coordinate (used for curve fitting)
/// 
#[derive(Clone, Copy)]
struct InkCoord {
    x: f32,
    y: f32,
    pressure: f32
}

impl InkCoord {
    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn pressure(&self) -> f32 { self.pressure }

    pub fn to_coord2(&self) -> (Coord2, f32) {
        (Coord2(self.x, self.y), self.pressure)
    }
}

impl<'a> From<&'a BrushPoint> for InkCoord {
    fn from(src: &'a BrushPoint) -> InkCoord {
        InkCoord {
            x: src.position.0,
            y: src.position.1,
            pressure: src.pressure
        }
    }
}

impl Add<InkCoord> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn add(self, rhs: InkCoord) -> InkCoord {
        InkCoord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            pressure: self.pressure + rhs.pressure
        }
    }
}

impl Sub<InkCoord> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn sub(self, rhs: InkCoord) -> InkCoord {
        InkCoord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            pressure: self.pressure - rhs.pressure
        }
    }
}

impl Mul<f32> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn mul(self, rhs: f32) -> InkCoord {
        InkCoord {
            x: self.x * rhs,
            y: self.y * rhs,
            pressure: self.pressure * rhs
        }
    }
}

impl Coordinate for InkCoord {
    #[inline]
    fn from_components(components: &[f32]) -> InkCoord {
        InkCoord { x: components[0], y: components[1], pressure: components[2] }
    }

    #[inline]
    fn origin() -> InkCoord {
        InkCoord { x: 0.0, y: 0.0, pressure: 0.0 }
    }

    #[inline]
    fn len() -> usize { 3 }

    #[inline]
    fn get(&self, index: usize) -> f32 { 
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.pressure,
            _ => panic!("InkCoord only has three components")
        }
    }

    fn from_biggest_components(p1: InkCoord, p2: InkCoord) -> InkCoord {
        InkCoord {
            x: f32::from_biggest_components(p1.x, p2.x),
            y: f32::from_biggest_components(p1.y, p2.y),
            pressure: f32::from_biggest_components(p1.pressure, p2.pressure)
        }
    }

    fn from_smallest_components(p1: InkCoord, p2: InkCoord) -> InkCoord {
        InkCoord {
            x: f32::from_smallest_components(p1.x, p2.x),
            y: f32::from_smallest_components(p1.y, p2.y),
            pressure: f32::from_smallest_components(p1.pressure, p2.pressure)
        }
    }

    #[inline]
    fn distance_to(&self, target: &InkCoord) -> f32 {
        let dist_x = target.x-self.x;
        let dist_y = target.y-self.y;
        let dist_p = target.pressure-self.pressure;

        f32::sqrt(dist_x*dist_x + dist_y*dist_y + dist_p*dist_p)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f32 {
        self.x*target.x + self.y*target.y + self.pressure*target.pressure
    }
}

///
/// Bezier curve using InkCoords
/// 
#[derive(Clone, Copy)]
struct InkCurve {
    pub start_point:    InkCoord,
    pub end_point:      InkCoord,
    pub control_points: (InkCoord, InkCoord)
}

impl InkCurve {
    pub fn to_curve(&self) -> bezier::Curve {
        bezier::Curve::from_points(self.start_point.to_coord2().0, self.end_point.to_coord2().0, self.control_points.0.to_coord2().0, self.control_points.1.to_coord2().0)
    }
}

impl BezierCurve for InkCurve {
    type Point = InkCoord;

    fn from_points(start: InkCoord, end: InkCoord, control_point1: InkCoord, control_point2: InkCoord) -> InkCurve {
        InkCurve {
            start_point:    start,
            end_point:      end,
            control_points: (control_point1, control_point2)
        }
    }

    #[inline]
    fn start_point(&self) -> InkCoord {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> InkCoord {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> (InkCoord, InkCoord) {
        self.control_points
    }
}

impl Brush for InkBrush {
    fn render_brush(&self, gc: &mut GraphicsPrimitives, points: &Vec<BrushPoint>) {
        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 1 {
            return;
        }

        // Convert points to ink points
        let ink_points: Vec<InkCoord> = points.iter().map(|point| InkCoord::from(point)).collect();

        // Pick points that are at least a certain distance apart to use for the fitting algorithm
        let mut distant_coords  = vec![];
        let mut last_point      = ink_points[0];

        distant_coords.push(last_point);
        for x in 1..ink_points.len() {
            if last_point.distance_to(&ink_points[x]) >= 4.0 {
                last_point = ink_points[x];
                distant_coords.push(last_point);
            }
        }

        // Fit these points to a curve
        let curve = InkCurve::fit_from_points(&distant_coords, 2.0);
        
        // Draw a simple line for this brush
        if let Some(curve) = curve {
            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            gc.new_path();
            
            let Coord2(x, y) = curve[0].start_point().to_coord2().0;
            gc.move_to(x, y);
            for curve_section in curve.iter().map(|section| section.to_curve()) {
                gc_draw_bezier(gc, &curve_section);
            }

            gc.stroke();
        }
    }
}
