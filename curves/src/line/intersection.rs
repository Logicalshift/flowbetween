use super::line::*;
use super::super::coordinate::*;

use std::f64;

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
    let (txmin, txmax) = if dx == 0.0 {
        // Parallel to the x axis
        (f64::INFINITY, f64::INFINITY)
    } else {
        ((xmin-x1)/dx, (xmax-x1)/dx)
    };

    let (tymin, tymax) = if dy == 0.0 {
        // Parallel to the y axis
        (f64::INFINITY, f64::INFINITY)
    } else {
        ((ymin-y1)/dy, (ymax-y1)/dy)
    };

    let tmin = txmin.min(tymin);
    let tmax = txmax.min(tymax);

    if tmin < 0.0 && tmax < 0.0 {
        // Outside the bounds
        None
    } else if tmin > 1.0 && tmax > 1.0 {
        // Outside the bounds
        None
    } else {
        // t values on the line range from 0-1
        let tmin = tmin.max(0.0).min(1.0);
        let tmax = tmax.max(0.0).min(1.0);

        // Create points from these components
        let p1 = L::Point::from_components(&[x1+dx*tmin, y1+dy*tmin]);
        let p2 = L::Point::from_components(&[x1+dx*tmax, y1+dy*tmax]);

        // This is the line clipped to these bounds
        Some(L::from_points(p1, p2))
    }
}
