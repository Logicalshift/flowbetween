use flo_animation::*;

///
/// A control point for the adjust tool
///
#[derive(Clone, Debug, PartialEq)]
pub (super) struct AdjustControlPoint {
    pub (super) owner:          ElementId,
    pub (super) index:          usize,
    pub (super) control_point:  ControlPoint
}

///
/// Identifier for a control point
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub (super) struct AdjustControlPointId {
    pub (super) owner:  ElementId,
    pub (super) index:  usize
}

impl From<&AdjustControlPoint> for AdjustControlPointId {
    fn from(cp: &AdjustControlPoint) -> AdjustControlPointId {
        AdjustControlPointId {
            owner: cp.owner,
            index: cp.index
        }
    }
}
