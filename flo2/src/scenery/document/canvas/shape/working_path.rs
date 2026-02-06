use super::super::point::*;
use super::path::*;
use super::working_point::*;

use flo_curves::bezier::path::*;
use flo_curves::geo::*;

use ::serde::*;

///
/// Represents a subpath of a shape on the canvas, used for working on a path in-memory
///
/// 'Precision' version, using 64-bit points that's intended for path arithmetic operations
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkingSubpath {
    /// Initial point of the path
    pub start_point: WorkingPoint,

    /// The actions that make up this path
    pub actions: Vec<WorkingPathAction>,
}

///
/// Actions that can be taken as part of a in memory working subpath
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WorkingPathAction {
    /// Line to a specific point
    Line(WorkingPoint),

    /// Quadratic bezier curve to the specified point
    QuadraticCurve { end: WorkingPoint, cp: WorkingPoint },

    /// Cubic bezier curve to a specific point
    CubicCurve { end: WorkingPoint, cp1: WorkingPoint, cp2: WorkingPoint },

    /// Closes the path (generating a line to the start point)
    Close,
}

impl WorkingPathAction {
    ///
    /// Returns the end point of this action, given the current point and start point of the path
    ///
    fn end_point(&self, start: WorkingPoint) -> WorkingPoint {
        match self {
            WorkingPathAction::Line(end)                  => *end,
            WorkingPathAction::QuadraticCurve { end, .. } => *end,
            WorkingPathAction::CubicCurve { end, .. }     => *end,
            WorkingPathAction::Close                      => start,
        }
    }

    ///
    /// Converts this action to a cubic bezier curve (cp1, cp2, end), given the current point and start point
    ///
    fn to_cubic(&self, current: WorkingPoint, start: WorkingPoint) -> (WorkingPoint, WorkingPoint, WorkingPoint) {
        match self {
            WorkingPathAction::Line(end) => {
                // Line becomes a cubic curve with control points at 1/3 and 2/3 along the line
                let cp1 = current + (*end - current) * (1.0 / 3.0);
                let cp2 = current + (*end - current) * (2.0 / 3.0);
                (cp1, cp2, *end)
            }

            WorkingPathAction::QuadraticCurve { end, cp } => {
                // Elevate quadratic to cubic: CP1 = P0 + 2/3*(CP - P0), CP2 = P1 + 2/3*(CP - P1)
                let cp1 = current + (*cp - current) * (2.0 / 3.0);
                let cp2 = *end + (*cp - *end) * (2.0 / 3.0);
                (cp1, cp2, *end)
            }

            WorkingPathAction::CubicCurve { end, cp1, cp2 } => {
                (*cp1, *cp2, *end)
            }

            WorkingPathAction::Close => {
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
    pub fn simplify(&self, current_point: WorkingPoint, max_error: f64) -> WorkingPathAction {
        match self {
            WorkingPathAction::CubicCurve { end, cp1, cp2 } => {
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
        start:      WorkingPoint,
        cp1:        WorkingPoint,
        cp2:        WorkingPoint,
        end:        WorkingPoint,
        max_error:  f64,
    ) -> Option<WorkingPathAction> {
        // Calculate the distance from each control point to the line (start -> end)
        let line_vec    = end - start;
        let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;

        // Handle degenerate case where start == end
        if line_len_sq < max_error * max_error {
            // Check if control points are also close to start
            if start.is_near_to(&cp1, max_error) && start.is_near_to(&cp2, max_error) {
                return Some(WorkingPathAction::Line(end));
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
            Some(WorkingPathAction::Line(end))
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
        start:      WorkingPoint,
        cp1:        WorkingPoint,
        cp2:        WorkingPoint,
        end:        WorkingPoint,
        max_error:  f64,
    ) -> Option<WorkingPathAction> {
        // Recover the quadratic control point from cp1: qcp = start + (cp1 - start) * 3/2
        let qcp_from_cp1 = start + (cp1 - start) * 1.5;

        // Recover the quadratic control point from cp2: qcp = end + (cp2 - end) * 3/2
        let qcp_from_cp2 = end + (cp2 - end) * 1.5;

        // Check if both recovered control points are the same (within tolerance)
        if qcp_from_cp1.is_near_to(&qcp_from_cp2, max_error) {
            // Use the average of the two recovered control points
            let qcp = WorkingPoint {
                x: (qcp_from_cp1.x + qcp_from_cp2.x) * 0.5,
                y: (qcp_from_cp1.y + qcp_from_cp2.y) * 0.5,
            };

            Some(WorkingPathAction::QuadraticCurve { end, cp: qcp })
        } else {
            None
        }
    }
}

impl Geo for WorkingSubpath {
    type Point = WorkingPoint;
}

///
/// Iterator over the curve points in a CanvasPrecisionSubpath, converting all actions to cubic bezier curves
///
pub struct CanvasPrecisionSubpathPointIter {
    start_point:   WorkingPoint,
    current_point: WorkingPoint,
    actions:       std::vec::IntoIter<WorkingPathAction>,
}

impl Iterator for CanvasPrecisionSubpathPointIter {
    type Item = (WorkingPoint, WorkingPoint, WorkingPoint);

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

impl BezierPath for WorkingSubpath {
    type PointIter = CanvasPrecisionSubpathPointIter;

    ///
    /// Retrieves the initial point of this path
    ///
    fn start_point(&self) -> WorkingPoint {
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

impl BezierPathFactory for WorkingSubpath {
    ///
    /// Creates a new instance of this path from a set of cubic bezier curve points
    ///
    fn from_points<FromIter: IntoIterator<Item = (WorkingPoint, WorkingPoint, WorkingPoint)>>(
        start_point: Self::Point,
        points: FromIter,
    ) -> Self {
        let actions = points
            .into_iter()
            .map(|(cp1, cp2, end)| WorkingPathAction::CubicCurve { end, cp1, cp2 })
            .collect();

        WorkingSubpath { start_point, actions }
    }
}

impl WorkingSubpath {
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
    pub fn simplify(&self, max_error: f64) -> WorkingSubpath {
        let mut current_point    = self.start_point;
        let mut simplified_actions = Vec::with_capacity(self.actions.len());

        for action in &self.actions {
            let simplified   = action.simplify(current_point, max_error);
            current_point    = action.end_point(self.start_point);
            simplified_actions.push(simplified);
        }

        WorkingSubpath {
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
    pub fn from_canvas_path(path: &CanvasPath) -> Vec<WorkingSubpath> {
        let mut subpaths = Vec::new();
        let mut current_start = WorkingPoint::from(path.start_point);
        let mut current_actions: Vec<WorkingPathAction> = Vec::new();

        for action in &path.actions {
            match action {
                CanvasPathV1Action::Move(point) => {
                    // Finish the current subpath if it has any actions
                    if !current_actions.is_empty() {
                        subpaths.push(WorkingSubpath {
                            start_point: current_start,
                            actions:     std::mem::take(&mut current_actions),
                        });
                    }
                    // Start a new subpath
                    current_start = WorkingPoint::from(*point);
                }

                CanvasPathV1Action::Close => {
                    current_actions.push(WorkingPathAction::Close);
                }

                CanvasPathV1Action::Line(end) => {
                    current_actions.push(WorkingPathAction::Line(WorkingPoint::from(*end)));
                }

                CanvasPathV1Action::QuadraticCurve { end, cp } => {
                    current_actions.push(WorkingPathAction::QuadraticCurve {
                        end: WorkingPoint::from(*end),
                        cp:  WorkingPoint::from(*cp),
                    });
                }

                CanvasPathV1Action::CubicCurve { end, cp1, cp2 } => {
                    current_actions.push(WorkingPathAction::CubicCurve {
                        end: WorkingPoint::from(*end),
                        cp1: WorkingPoint::from(*cp1),
                        cp2: WorkingPoint::from(*cp2),
                    });
                }
            }
        }

        // Add the final subpath if it has any actions
        if !current_actions.is_empty() {
            subpaths.push(WorkingSubpath {
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
    pub fn to_canvas_path(subpaths: &[WorkingSubpath]) -> CanvasPath {
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
                    WorkingPathAction::Close => CanvasPathV1Action::Close,

                    WorkingPathAction::Line(end) => {
                        CanvasPathV1Action::Line(CanvasPoint::from(*end))
                    }

                    WorkingPathAction::QuadraticCurve { end, cp } => {
                        CanvasPathV1Action::QuadraticCurve {
                            end: CanvasPoint::from(*end),
                            cp:  CanvasPoint::from(*cp),
                        }
                    }

                    WorkingPathAction::CubicCurve { end, cp1, cp2 } => {
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
