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
