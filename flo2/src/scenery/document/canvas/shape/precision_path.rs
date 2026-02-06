use super::super::point::*;
use super::path::*;
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

    ///
    /// Attempts to simplify a cubic curve to a line or quadratic curve if it fits within the given error bound.
    /// Returns the simplified action, or the original action if no simplification is possible.
    ///
    /// For a cubic curve starting at `current_point`:
    /// - Returns a Line if both control points lie on the line between start and end within `max_error`
    /// - Returns a QuadraticCurve if the cubic can be represented as a quadratic within `max_error`
    /// - Otherwise returns the original CubicCurve
    ///
    pub fn simplify(&self, current_point: CanvasPrecisionPoint, max_error: f64) -> CanvasPrecisionPathAction {
        match self {
            CanvasPrecisionPathAction::CubicCurve { end, cp1, cp2 } => {
                // Try to simplify to a line first
                if let Some(line) = Self::try_simplify_to_line(current_point, *cp1, *cp2, *end, max_error) {
                    return line;
                }

                // Try to simplify to a quadratic
                if let Some(quad) = Self::try_simplify_to_quadratic(current_point, *cp1, *cp2, *end, max_error) {
                    return quad;
                }

                // No simplification possible
                self.clone()
            }

            // Other action types are already in their simplest form
            _ => self.clone(),
        }
    }

    ///
    /// Checks if a cubic curve is actually a straight line (control points lie on the line between endpoints).
    ///
    fn try_simplify_to_line(
        start:      CanvasPrecisionPoint,
        cp1:        CanvasPrecisionPoint,
        cp2:        CanvasPrecisionPoint,
        end:        CanvasPrecisionPoint,
        max_error:  f64,
    ) -> Option<CanvasPrecisionPathAction> {
        // Calculate the distance from each control point to the line (start -> end)
        let line_vec    = end - start;
        let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;

        // Handle degenerate case where start == end
        if line_len_sq < max_error * max_error {
            // Check if control points are also close to start
            if start.is_near_to(&cp1, max_error) && start.is_near_to(&cp2, max_error) {
                return Some(CanvasPrecisionPathAction::Line(end));
            }
            return None;
        }

        let line_len = line_len_sq.sqrt();

        // Distance from point to line using cross product: |((p - start) x line_vec)| / |line_vec|
        let cp1_to_start = cp1 - start;
        let cp2_to_start = cp2 - start;

        // 2D cross product gives signed area, divide by line length for distance
        let cp1_cross    = cp1_to_start.x * line_vec.y - cp1_to_start.y * line_vec.x;
        let cp2_cross    = cp2_to_start.x * line_vec.y - cp2_to_start.y * line_vec.x;

        let cp1_distance = cp1_cross.abs() / line_len;
        let cp2_distance = cp2_cross.abs() / line_len;

        if cp1_distance <= max_error && cp2_distance <= max_error {
            Some(CanvasPrecisionPathAction::Line(end))
        } else {
            None
        }
    }

    ///
    /// Checks if a cubic curve can be represented as a quadratic curve.
    ///
    /// A cubic that was elevated from a quadratic has:
    /// - cp1 = start + 2/3 * (qcp - start)
    /// - cp2 = end + 2/3 * (qcp - end)
    ///
    /// Solving for qcp from both equations should give the same point if it's truly a quadratic.
    ///
    fn try_simplify_to_quadratic(
        start:      CanvasPrecisionPoint,
        cp1:        CanvasPrecisionPoint,
        cp2:        CanvasPrecisionPoint,
        end:        CanvasPrecisionPoint,
        max_error:  f64,
    ) -> Option<CanvasPrecisionPathAction> {
        // Recover the quadratic control point from cp1: qcp = start + (cp1 - start) * 3/2
        let qcp_from_cp1 = start + (cp1 - start) * 1.5;

        // Recover the quadratic control point from cp2: qcp = end + (cp2 - end) * 3/2
        let qcp_from_cp2 = end + (cp2 - end) * 1.5;

        // Check if both recovered control points are the same (within tolerance)
        if qcp_from_cp1.is_near_to(&qcp_from_cp2, max_error) {
            // Use the average of the two recovered control points
            let qcp = CanvasPrecisionPoint {
                x: (qcp_from_cp1.x + qcp_from_cp2.x) * 0.5,
                y: (qcp_from_cp1.y + qcp_from_cp2.y) * 0.5,
            };

            Some(CanvasPrecisionPathAction::QuadraticCurve { end, cp: qcp })
        } else {
            None
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

impl CanvasPrecisionSubpath {
    ///
    /// Simplifies all curves in this path by converting cubic curves to lines or quadratic curves
    /// where they fit within the given error bound.
    ///
    /// This is useful for reducing the complexity of paths generated by operations like boolean
    /// path operations, which typically output all curves as cubics even when simpler representations
    /// would suffice.
    ///
    /// Returns a new path with simplified curves.
    ///
    pub fn simplify(&self, max_error: f64) -> CanvasPrecisionSubpath {
        let mut current_point    = self.start_point;
        let mut simplified_actions = Vec::with_capacity(self.actions.len());

        for action in &self.actions {
            let simplified   = action.simplify(current_point, max_error);
            current_point    = action.end_point(self.start_point);
            simplified_actions.push(simplified);
        }

        CanvasPrecisionSubpath {
            start_point: self.start_point,
            actions:     simplified_actions,
        }
    }

    ///
    /// Simplifies all curves in this path in place.
    ///
    pub fn simplify_in_place(&mut self, max_error: f64) {
        let mut current_point = self.start_point;

        for action in &mut self.actions {
            let simplified = action.simplify(current_point, max_error);
            current_point  = action.end_point(self.start_point);
            *action        = simplified;
        }
    }

    ///
    /// Converts a CanvasPath (which may contain multiple subpaths via Move actions) into a
    /// Vec of CanvasPrecisionSubpath (one per subpath).
    ///
    /// This splits the path at each Move action and converts coordinates from f32 to f64.
    ///
    pub fn from_canvas_path(path: &CanvasPath) -> Vec<CanvasPrecisionSubpath> {
        let mut subpaths = Vec::new();
        let mut current_start = CanvasPrecisionPoint::from(path.start_point);
        let mut current_actions: Vec<CanvasPrecisionPathAction> = Vec::new();

        for action in &path.actions {
            match action {
                CanvasPathV1Action::Move(point) => {
                    // Finish the current subpath if it has any actions
                    if !current_actions.is_empty() {
                        subpaths.push(CanvasPrecisionSubpath {
                            start_point: current_start,
                            actions:     std::mem::take(&mut current_actions),
                        });
                    }
                    // Start a new subpath
                    current_start = CanvasPrecisionPoint::from(*point);
                }

                CanvasPathV1Action::Close => {
                    current_actions.push(CanvasPrecisionPathAction::Close);
                }

                CanvasPathV1Action::Line(end) => {
                    current_actions.push(CanvasPrecisionPathAction::Line(CanvasPrecisionPoint::from(*end)));
                }

                CanvasPathV1Action::QuadraticCurve { end, cp } => {
                    current_actions.push(CanvasPrecisionPathAction::QuadraticCurve {
                        end: CanvasPrecisionPoint::from(*end),
                        cp:  CanvasPrecisionPoint::from(*cp),
                    });
                }

                CanvasPathV1Action::CubicCurve { end, cp1, cp2 } => {
                    current_actions.push(CanvasPrecisionPathAction::CubicCurve {
                        end: CanvasPrecisionPoint::from(*end),
                        cp1: CanvasPrecisionPoint::from(*cp1),
                        cp2: CanvasPrecisionPoint::from(*cp2),
                    });
                }
            }
        }

        // Add the final subpath if it has any actions
        if !current_actions.is_empty() {
            subpaths.push(CanvasPrecisionSubpath {
                start_point: current_start,
                actions:     current_actions,
            });
        }

        subpaths
    }

    ///
    /// Converts a Vec of CanvasPrecisionSubpath back into a single CanvasPath.
    ///
    /// Subpaths after the first are joined with Move actions. Coordinates are converted from f64 to f32.
    ///
    pub fn to_canvas_path(subpaths: &[CanvasPrecisionSubpath]) -> CanvasPath {
        if subpaths.is_empty() {
            return CanvasPath {
                start_point: CanvasPoint { x: 0.0, y: 0.0 },
                actions:     Vec::new(),
            };
        }

        let start_point = CanvasPoint::from(subpaths[0].start_point);
        let mut actions = Vec::new();

        for (idx, subpath) in subpaths.iter().enumerate() {
            // Add a Move action for subpaths after the first
            if idx > 0 {
                actions.push(CanvasPathV1Action::Move(CanvasPoint::from(subpath.start_point)));
            }

            // Convert all actions in this subpath
            for action in &subpath.actions {
                actions.push(match action {
                    CanvasPrecisionPathAction::Close => CanvasPathV1Action::Close,

                    CanvasPrecisionPathAction::Line(end) => {
                        CanvasPathV1Action::Line(CanvasPoint::from(*end))
                    }

                    CanvasPrecisionPathAction::QuadraticCurve { end, cp } => {
                        CanvasPathV1Action::QuadraticCurve {
                            end: CanvasPoint::from(*end),
                            cp:  CanvasPoint::from(*cp),
                        }
                    }

                    CanvasPrecisionPathAction::CubicCurve { end, cp1, cp2 } => {
                        CanvasPathV1Action::CubicCurve {
                            end: CanvasPoint::from(*end),
                            cp1: CanvasPoint::from(*cp1),
                            cp2: CanvasPoint::from(*cp2),
                        }
                    }
                });
            }
        }

        CanvasPath { start_point, actions }
    }
}
