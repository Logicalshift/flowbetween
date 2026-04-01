use super::adjust_control_point::*;

///
/// Describes a point on a curve
///
#[derive(Clone, Debug, PartialEq)]
pub (super) struct AdjustEdgePoint {
    /// The start point of the bezier curve section
    pub (super) start_point: AdjustControlPointId,

    // The end point of the bezier curve section
    pub (super) end_point: AdjustControlPointId,

    /// The t-value of the point that's nearest to the cursor
    pub (super) t: f64,

    /// The distance to the curve
    pub (super) distance: f64
}
