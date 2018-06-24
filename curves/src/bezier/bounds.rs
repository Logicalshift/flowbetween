use super::basis::*;
use super::super::geo::*;
use super::super::coordinate::*;

///
/// Finds the t values of the extremities of a curve (these are the points at which
/// the x or y value is at a minimum or maximum)
/// 
pub fn find_extremities<Point: Coordinate>(w1: Point, w2: Point, w3: Point, w4: Point) -> Vec<f64> {
    // The 't' values where this curve has extremities we need to examine
    let mut t_extremes = vec![1.0];

    // The derivative is a quadratic function, so we can compute the locations of these (t values) by solving the quadratic formula for them
    for component_index in 0..Point::len() {
        // Fetch the parameters for this component
        let p1 = w1.get(component_index);
        let p2 = w2.get(component_index);
        let p3 = w3.get(component_index);
        let p4 = w4.get(component_index);

        // Compute the bezier coefficients
        let a = (-p1 + p2*3.0 - p3*3.0 + p4)*3.0;
        let b = (p1 - p2*2.0 + p3)*6.0;
        let c = (p2 - p1)*3.0;

        // Extremities are points at which the curve has a 0 gradient (in any of its dimensions)
        let root1 = (-b + f64::sqrt(b*b - a*c*4.0)) / (a*2.0);
        let root2 = (-b - f64::sqrt(b*b - a*c*4.0)) / (a*2.0);

        if root1 > 0.0 && root1 < 1.0 { t_extremes.push(root1); }
        if root2 > 0.0 && root2 < 1.0 { t_extremes.push(root2); }

        // We also solve for the second derivative
        let aa = 2.0*(b-a);
        let bb = 2.0*(c-a);

        // Solve for a'*t+b = 0 (0-b/a')
        if aa != 0.0 {
            let root3 = -bb/aa;
            if root3 > 0.0 && root3 < 1.0 {
                t_extremes.push(root3);
            }
        }
    }

    t_extremes
}

///
/// Finds the upper and lower points in a cubic curve's bounding box
/// 
pub fn bounding_box4<Point: Coordinate, Bounds: BoundingBox<Point=Point>>(w1: Point, w2: Point, w3: Point, w4: Point) -> Bounds {
    // The 't' values where this curve has extremities we need to examine
    let t_extremes = find_extremities(w1, w2, w3, w4);

    // Start with the point at 0,0 as the minimum position
    let mut min_pos = de_casteljau4(0.0, w1, w2, w3, w4);
    let mut max_pos = min_pos;

    for t in t_extremes {
        let point = de_casteljau4(t, w1, w2, w3, w4);

        min_pos = Point::from_smallest_components(min_pos, point);
        max_pos = Point::from_biggest_components(max_pos, point);
    }

    Bounds::from_min_max(min_pos, max_pos)
}
