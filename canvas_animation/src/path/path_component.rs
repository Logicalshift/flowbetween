use super::animation_path::*;

use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::path::*;

///
/// A path component represents a single closed path 
///
#[derive(Clone)]
pub struct PathComponent<'a> {
    /// The slice of path operations for this path component. The first item in the slice is the move operation, and the last is the item prior to the next move operation
    slice: &'a [PathOp]
}

///
/// Iterates through a path component
///
pub struct PathComponentIterator<'a> {
    /// The component that this is iterating over
    slice: &'a [PathOp],

    /// The index into the slice
    idx: usize
}

impl<'a> PathComponent<'a> {
    ///
    /// Divides an animation path into path components
    ///
    pub fn from_path(path: &'a AnimationPath) -> Vec<PathComponent<'a>> {
        // The result of this operation
        let mut path_components = vec![];

        // Short-circuit if the path is empty
        if path.path.len() == 0 {
            return path_components;
        }

        // The index of the 'Move' instruction that started the current path
        let mut initial_index   = 0;

        // Find the end component of each path
        for path_index in 1..path.path.len() {
            if let PathOp::Move(_, _) = &path.path[path_index] {
                // Split a component at this point
                let component = PathComponent { slice: &path.path[initial_index..path_index] };
                path_components.push(component);

                // This move instruction is the start of the next path
                initial_index = path_index;
            }
        }

        // The last component might not be broken off with a move
        if initial_index + 1 < path.path.len() {
            let component = PathComponent { slice: &path.path[initial_index..path.path.len()] };
            path_components.push(component);
        }

        path_components
    }
}

///
/// Returns the position of a particular path op
///
#[inline]
fn position_of(op: &PathOp) -> Coord2 {
    match op {
        PathOp::NewPath                 => { Coord2(0.0, 0.0) }
        PathOp::ClosePath               => { Coord2(0.0, 0.0) }
        PathOp::Move(x, y)              => { Coord2(*x as _, *y as _) }
        PathOp::Line(x, y)              => { Coord2(*x as _, *y as _) }
        PathOp::BezierCurve(_, (x, y))  => { Coord2(*x as _, *y as _) }
    }
}

impl<'a> Iterator for PathComponentIterator<'a> {
    type Item = (Coord2, Coord2, Coord2);

    fn next(&mut self) -> Option<Self::Item> {
        const TINY_DISTANCE: f64    = 0.00001;

        // Advance the position (idx 0 is the start point so we always skip that, additionally we always know that idx-1 exists)
        self.idx += 1;

        // See if we've reached the end of the slice
        if self.idx >= self.slice.len() {
            // No more items in this path
            None
        } else {
            // Path coordinate depends on the path type
            match self.slice[self.idx] {
                PathOp::Move(_, _)  => { None },
                PathOp::NewPath     => { None },

                PathOp::BezierCurve(((cp1x, cp1y), (cp2x, cp2y)), (x, y)) => {
                    Some((Coord2(cp1x as _, cp1y as _), Coord2(cp2x as _, cp2y as _), Coord2(x as _, y as _)))
                }

                PathOp::Line(x, y)  => {
                    // Generate control points for a bezier curve representing a line
                    let final_point     = Coord2(x as _, y as _);
                    let initial_point   = position_of(&self.slice[self.idx-1]);

                    let diff            = final_point - initial_point;
                    let cp1             = initial_point + (diff * (1.0/3.0));
                    let cp2             = initial_point + (diff * (2.0/3.0));

                    Some((cp1, cp2, final_point))
                }

                PathOp::ClosePath   => {
                    // Line between last point and first point, or None if they are the same
                    let final_point     = position_of(&self.slice[0]);
                    let initial_point   = position_of(&self.slice[self.idx-1]);

                    let diff            = final_point - initial_point;
                    let diff_sq         = (diff.x()*diff.x()) + (diff.y()*diff.y());

                    if diff_sq <= TINY_DISTANCE*TINY_DISTANCE {
                        // Path already closed
                        None
                    } else {
                        // Control points for a line closing this path
                        let cp1             = initial_point + (diff * (1.0/3.0));
                        let cp2             = initial_point + (diff * (2.0/3.0));

                        Some((cp1, cp2, final_point))
                    }
                }
            }
        }
    }
}

impl<'a> Geo for PathComponent<'a> {
    type Point = Coord2;
}

impl<'a> BezierPath for PathComponent<'a> {
    type PointIter = PathComponentIterator<'a>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> Self::Point {
        position_of(&self.slice[0])
    }

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    fn points(&self) -> Self::PointIter {
        PathComponentIterator {
            slice:  self.slice,
            idx:    0
        }
    }
}
