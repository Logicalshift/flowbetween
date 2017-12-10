///
/// Trait implemented by things representing a cubic bezier curve
/// 
pub trait BezierCurve {
    ///
    /// Creates a new bezier curve of the same type from some points
    /// 
    fn from_points(start: (f32, f32), end: (f32, f32), control_point1: (f32, f32), control_point2: (f32, f32)) -> Self;

    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> (f32, f32);

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> (f32, f32);

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> ((f32, f32), (f32, f32));
}

///
/// Represents a Bezier curve
/// 
pub struct Curve {
    pub start_point:    (f32, f32),
    pub end_point:      (f32, f32),
    pub control_points: ((f32, f32), (f32, f32))
}

impl BezierCurve for Curve {
    fn from_points(start: (f32, f32), end: (f32, f32), control_point1: (f32, f32), control_point2: (f32, f32)) -> Curve {
        Curve {
            start_point:    start,
            end_point:      end,
            control_points: (control_point1, control_point2)
        }
    }

    #[inline]
    fn start_point(&self) -> (f32, f32) {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> (f32, f32) {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> ((f32, f32), (f32, f32)) {
        self.control_points
    }
}
