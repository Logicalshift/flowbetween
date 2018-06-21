use super::line::*;
use super::super::coordinate::*;

///
/// Returns the point at which two lines intersect (if they intersect)
/// 
/// Only the 2-dimensional form is supported at the moment (lines are much less likely to intersect
/// in higher dimensions)
/// 
pub fn line_intersects_line<L: Line>(line1: &L, line2: &L) -> Option<L::Point> 
where L::Point: Coordinate2D {
    let line1_points = line1.points();
    let line2_points = line2.points();

    let ((x1, y1), (x2, y2)) = (line1_points.0.coords(), line1_points.1.coords());
    let ((x3, y3), (x4, y4)) = (line2_points.0.coords(), line2_points.1.coords());

    let ua = ((x4-x3)*(y1-y3) - (y4-y3)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
    let ub = ((x2-x1)*(y1-y3) - (y2-y1)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));

    if ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0 {
        Some(L::Point::from_components(&[
            x1+(ua*(x2-x1)), 
            y1+(ua*(y2-y1))
        ]))
    } else {
        None
    }
}

///
/// Determines if a 2D line has intersected a bounding box (and returns the intersection if it exists)
/// 
pub fn line_clip_to_bounds<L: Line>(line: &L, bounds: &(L::Point, L::Point)) -> Option<L>
where L::Point: Coordinate2D {
    // Fetch the points for the line
    let line_points             = line.points();
    let ((x1, y1), (x2, y2))    = (line_points.0.coords(), line_points.1.coords());
    let (dx, dy)                = (x2-x1, y2-y1);

    // ... and the points for the bounding rectangle
    let (xmin, ymin)            = (bounds.0.x().min(bounds.1.x()), bounds.0.y().min(bounds.1.y()));
    let (xmax, ymax)            = (bounds.0.x().max(bounds.1.x()), bounds.0.y().max(bounds.1.y()));

    // Our line can be described as '(x1+t*dx, y1+t*dy)' where 0 <= t <= 1
    // We want to solve for the edges, eg (xmin=x1+tmin*dx => txmin=(xmin-x1)/dx)
    let delta   = [-dx, dx, -dy, dy];
    let edge    = [x1-xmin, xmax-x1, y1-ymin, ymax-y1];

    // t1 and t2 are the points on the line. Initially they describe the entire line (t=0 -> t=1)
    let mut t1  = 0.0;
    let mut t2  = 1.0;

    // Clip against each of the 4 edges in turn
    for (delta, edge) in delta.into_iter().zip(edge.into_iter()) {
        if delta == &0.0 {
            // Line is parallel to this edge
            if edge < &0.0 {
                // Line is outside of the rectangle
                return None;
            }
        } else {
            // Compute the 't' value where the line intersects this edge
            let t = edge/delta;

            if delta < &0.0 && t1 < t {
                // Start of the line is clipped against this edge (if the delta value is <0 the start is closer to this edge)
                t1 = t;
            } else if delta > &0.0 && t2 > t {
                // End of the line is clipped against this edge (if the delta value is >0 the end is closer to this edge)
                t2 = t;
            }
        }
    }

    if t1 > t2 || t1 > 1.0 || t2 < 0.0 {
        // Line does not intersect rectangle (intersection is entirely outside the rectangle or ends before it starts)
        None
    } else {
        // Line intersects the rectangle. t1 and t2 indicate where
        let p1 = L::Point::from_components(&[x1 + t1*dx, y1 + t1*dy]);
        let p2 = L::Point::from_components(&[x1 + t2*dx, y1 + t2*dy]);

        Some(L::from_points(p1, p2))
    }
}
