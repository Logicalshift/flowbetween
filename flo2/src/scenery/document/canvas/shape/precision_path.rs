use super::precision_point::*;

use flo_curves::bezier::path::*;
use flo_curves::geo::*;

use ::serde::*;

///
/// Represents a subpath of a shape on the canvas
///
/// 'Precision' version, using 64-bit points that's intended for path arithmetic operations
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPrecisionSubpath {
    /// Initial point of the path
    pub start_point: CanvasPrecisionPoint,

    /// The actions that make up this path
    pub actions: Vec<CanvasPrecisionPathAction>,
}

///
/// Actions that can be taken as part of a precision subpath
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CanvasPrecisionPathAction {
    /// Line to a specific point
    Line(CanvasPrecisionPoint),

    /// Quadratic bezier curve to the specified point
    QuadraticCurve { end: CanvasPrecisionPoint, cp: CanvasPrecisionPoint },

    /// Cubic bezier curve to a specific point
    CubicCurve { end: CanvasPrecisionPoint, cp1: CanvasPrecisionPoint, cp2: CanvasPrecisionPoint },

    /// Closes the path (generating a line to the start point)
    Close,
}

impl CanvasPrecisionPathAction {
    ///
    /// Returns the end point of this action, given the current point and start point of the path
    ///
    fn end_point(&self, start: CanvasPrecisionPoint) -> CanvasPrecisionPoint {
        match self {
            CanvasPrecisionPathAction::Line(end)                  => *end,
            CanvasPrecisionPathAction::QuadraticCurve { end, .. } => *end,
            CanvasPrecisionPathAction::CubicCurve { end, .. }     => *end,
            CanvasPrecisionPathAction::Close                      => start,
        }
    }

    ///
    /// Converts this action to a cubic bezier curve (cp1, cp2, end), given the current point and start point
    ///
    fn to_cubic(&self, current: CanvasPrecisionPoint, start: CanvasPrecisionPoint) -> (CanvasPrecisionPoint, CanvasPrecisionPoint, CanvasPrecisionPoint) {
        match self {
            CanvasPrecisionPathAction::Line(end) => {
                // Line becomes a cubic curve with control points at 1/3 and 2/3 along the line
                let cp1 = current + (*end - current) * (1.0 / 3.0);
                let cp2 = current + (*end - current) * (2.0 / 3.0);
                (cp1, cp2, *end)
            }

            CanvasPrecisionPathAction::QuadraticCurve { end, cp } => {
                // Elevate quadratic to cubic: CP1 = P0 + 2/3*(CP - P0), CP2 = P1 + 2/3*(CP - P1)
                let cp1 = current + (*cp - current) * (2.0 / 3.0);
                let cp2 = *end + (*cp - *end) * (2.0 / 3.0);
                (cp1, cp2, *end)
            }

            CanvasPrecisionPathAction::CubicCurve { end, cp1, cp2 } => {
                (*cp1, *cp2, *end)
            }

            CanvasPrecisionPathAction::Close => {
                // Close becomes a line back to start
                let cp1 = current + (start - current) * (1.0 / 3.0);
                let cp2 = current + (start - current) * (2.0 / 3.0);
                (cp1, cp2, start)
            }
        }
    }
}

impl Geo for CanvasPrecisionSubpath {
    type Point = CanvasPrecisionPoint;
}

///
/// Iterator over the curve points in a CanvasPrecisionSubpath, converting all actions to cubic bezier curves
///
pub struct CanvasPrecisionSubpathPointIter {
    start_point:   CanvasPrecisionPoint,
    current_point: CanvasPrecisionPoint,
    actions:       std::vec::IntoIter<CanvasPrecisionPathAction>,
}

impl Iterator for CanvasPrecisionSubpathPointIter {
    type Item = (CanvasPrecisionPoint, CanvasPrecisionPoint, CanvasPrecisionPoint);

    fn next(&mut self) -> Option<Self::Item> {
        self.actions.next().map(|action| {
            let cubic           = action.to_cubic(self.current_point, self.start_point);
            self.current_point  = action.end_point(self.start_point);
            cubic
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.actions.size_hint()
    }
}

impl ExactSizeIterator for CanvasPrecisionSubpathPointIter {
    #[inline]
    fn len(&self) -> usize {
        self.actions.len()
    }
}

impl BezierPath for CanvasPrecisionSubpath {
    type PointIter = CanvasPrecisionSubpathPointIter;

    ///
    /// Retrieves the initial point of this path
    ///
    fn start_point(&self) -> CanvasPrecisionPoint {
        self.start_point
    }

    ///
    /// Retrieves an iterator over the curve points in this path.
    /// Each item is (cp1, cp2, end) representing a cubic bezier curve.
    /// All action types are converted to cubic curves.
    ///
    fn points(&self) -> Self::PointIter {
        CanvasPrecisionSubpathPointIter {
            start_point:   self.start_point,
            current_point: self.start_point,
            actions:       self.actions.clone().into_iter(),
        }
    }
}

impl BezierPathFactory for CanvasPrecisionSubpath {
    ///
    /// Creates a new instance of this path from a set of cubic bezier curve points
    ///
    fn from_points<FromIter: IntoIterator<Item = (CanvasPrecisionPoint, CanvasPrecisionPoint, CanvasPrecisionPoint)>>(
        start_point: Self::Point,
        points: FromIter,
    ) -> Self {
        let actions = points
            .into_iter()
            .map(|(cp1, cp2, end)| CanvasPrecisionPathAction::CubicCurve { end, cp1, cp2 })
            .collect();

        CanvasPrecisionSubpath { start_point, actions }
    }
}
