///
/// Represents a control point in a vector element
/// 
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ControlPoint {
    /// Represents a point on a bezier curve
    BezierPoint(f32, f32),

    /// Represents a bezier control point
    BezierControlPoint(f32, f32)
}

impl ControlPoint {
    ///
    /// Returns the x, y position of this control point
    /// 
    pub fn position(&self) -> (f32, f32) {
        use self::ControlPoint::*;

        match self {
            BezierPoint(x, y)           => (*x, *y),
            BezierControlPoint(x, y)    => (*x, *y)
        }
    }
}
