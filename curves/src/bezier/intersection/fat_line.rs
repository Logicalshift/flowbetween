use super::super::curve::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use std::f64;

///
/// A 'fat line' is a line with a width. It's used in bezier intersection algorithms,
/// in particular the clipping algorithm described by Sederberg and Nishita
/// 
pub struct FatLine {
    /// The distance from the line to the upper part of the 'fat line'
    d_min: f64,

    /// The distance from the line to the lower part of the 'fat line'
    d_max: f64,

    /// The coefficients (a, b, c) in the equation ax+bx+c (where a^2+b^2 = 0)
    coeff: (f64, f64, f64)
}

impl FatLine {
    ///
    /// Returns the distance between the point and the central line
    /// 
    #[inline]
    pub fn distance<Point: Coordinate2D>(&self, point: &Point) -> f64 {
        let (a, b, c) = self.coeff;
        a*point.x() + b*point.y() + c
    }

    ///
    /// Given a bezier curve, returns another curve whose X axis is the distance
    /// from the central line and the Y axis varies from 0 to 1, with a uniform
    /// distribution of t values.
    /// 
    /// This is used in the bezier clipping algorithm to discover where a bezier
    /// curve clips against this line.
    /// 
    pub fn distance_curve<FromCurve: BezierCurve, ToCurve: BezierCurveFactory<Point=FromCurve::Point>>(&self, curve: &FromCurve) -> ToCurve
    where FromCurve::Point: Coordinate2D {
        let (cp1, cp2)  = curve.control_points();

        let start       = FromCurve::Point::from_components(&[self.distance(&curve.start_point()), 0.0]);
        let end         = FromCurve::Point::from_components(&[self.distance(&curve.end_point()), 1.0]);
        let cp1         = FromCurve::Point::from_components(&[self.distance(&cp1), 1.0/3.0]);
        let cp2         = FromCurve::Point::from_components(&[self.distance(&cp2), 2.0/3.0]);

        ToCurve::from_points(start, end, cp1, cp2)
    }

    ///
    /// Returns the convex hull of a curve returned by distance_curve
    /// 
    /// We can use some of the properties of the distance_curve to simplify how this
    /// is worked out (specifically, we know the points are sorted vertically already
    /// so we only need to know if the two control points are on the same side or not)
    /// 
    fn distance_curve_convex_hull<C: BezierCurve>(distance_curve: &C) -> Vec<C::Point> 
    where C::Point: Coordinate2D {
        // Read the points from the curve
        let start       = distance_curve.start_point();
        let (cp1, cp2)  = distance_curve.control_points();
        let end         = distance_curve.end_point();

        // Compute the x component of the distances of cp1 and cp2 from the central line defined by start->end
        // These are the m and c values for y=mx+c assuming that start.y() = 0 and end.y() = 1 which is true for the distance curve
        let m = end.x()-start.x();
        let c = start.x();

        let dx1 = cp1.x() - (m*(1.0/3.0)+c);
        let dx2 = cp2.x() - (m*(2.0/3.0)+c);

        // If they have the same sign, they're on the same side
        let on_same_side = dx1*dx2 >= 0.0;

        // Ordering on the convex hull depends only on if cp1 and cp2 are on the same side or not
        if on_same_side {
            // cp1 or cp2 might be inside the hull
            let dist_ratio = dx1/dx2;

            if dist_ratio >= 2.0 {
                // cp2 is in the hull (between the line cp1->end and start->end)
                vec![start, cp1, end]
            } else if dist_ratio <= 0.5 {
                // cp1 is in the hull (between the line cp2->end and start->end)
                vec![start, cp2, end]
            } else {
                // All points are on the hull
                vec![start, cp1, cp2, end]
            }
        } else {
            // It's not possible to have a point inside the hull
            vec![start, cp1, end, cp2]
        }
    }

    ///
    /// Rounds values very close to 0 or 1 to 0 or 1
    /// 
    #[inline]
    fn round_y_value(y: f64) -> f64 {
        if y < 0.0 && y > -0.001 {
            0.0
        } else if y > 1.0 && y < 1.001 {
            1.0
        } else {
            y
        }
    }

    ///
    /// Given an x pos on a line, solves for the y point
    /// 
    #[inline]
    fn solve_line_y<Point: Coordinate2D>((x1, x2): (f64, f64), (p1, p2): (&Point, &Point)) -> (Option<f64>, Option<f64>) {
        let min_x = p1.x().min(p2.x());
        let max_x = p1.x().max(p2.x());

        let m = (p2.y()-p1.y())/(p2.x()-p1.x());
        let c = p1.y() - m * p1.x();

        let y1 = if x1 >= min_x && x1 <= max_x { Some(Self::round_y_value(m*x1 + c)) } else { None };
        let y2 = if x2 >= min_x && x2 <= max_x { Some(Self::round_y_value(m*x2 + c)) } else { None };

        (y1, y2)
    }

    ///
    /// Clips the curve against this fat line. This attempts to find two new t values such
    /// that values outside that range are guaranteed not to lie within this fat line.
    /// 
    /// (This doesn't guarantee that the t values lie precisely within the line, though it's
    /// usually possible to iterate to improve the precision of the match)
    /// 
    pub fn clip_t<C: BezierCurve>(&self, curve: &C) -> Option<(f64, f64)> 
    where C::Point: Coordinate2D {
        // The 'distance' curve is a bezier curve where 'x' is the distance to the central line from the curve and 'y' is the t value where that distance occurs
        let distance_curve          = self.distance_curve::<_, Curve<C::Point>>(curve);

        // The convex hull encloses the distance curve, and can be used to find the y values where it's between d_min and d_max
        // As y=t due to how we construct the distance curve these are also the t values
        // We make use of the fact that the hull always has the start point at the start
        let distance_convex_hull    = Self::distance_curve_convex_hull(&distance_curve);

        // To solve for t, we need to find where the two edge lines cross d_min and d_max
        let num_points  = distance_convex_hull.len();
        let mut t1      = f64::MAX;
        let mut t2      = f64::MIN;
        let d_min       = self.d_min;
        let d_max       = self.d_max;
        for idx in 0..num_points {
            // Solve where this part of the convex hull crosses this line
            let (p1, p2)    = (&distance_convex_hull[idx], &distance_convex_hull[(idx+1)%num_points]);
            let hull_line   = (p1, p2);
            let (t1a, t2a)  = Self::solve_line_y((d_min, d_max), hull_line);

            // The y axis indicates where the hull crosses from inside to outside the fat line
            if let Some(t1a) = t1a {
                // Line crossed d_min
                if t1a >= 0.0 && t1a <= 1.0 { 
                    t1 = t1.min(t1a);
                    t2 = t2.max(t1a);
                }
            }
            
            if let Some(t2a) = t2a {
                // Line crossed d_max
                if t2a >= 0.0 && t2a <= 1.0 { 
                    t1 = t1.min(t2a);
                    t2 = t2.max(t2a);
                }
            }

            // If the start or end point is inside the fat line then it is also within the clipping area
            if p1.x() <= d_max && p1.x() >= d_min { t1 = t1.min(p1.y()); t2 = t2.max(p1.y()); }
            if p2.x() <= d_max && p2.x() >= d_min { t1 = t1.min(p2.y()); t2 = t2.max(p2.y()); }
        }

        if t1 > t2 {
            if t1 == f64::MAX {
                if t2 != f64::MIN {
                    // Only clipped against upper portion of the hull
                    Some((0.0, t2))
                } else {
                    // No part of the hull crossed the line (either entirely inside or outside)
                    let hull_x      = distance_convex_hull.into_iter().map(|p| p.x()).collect::<Vec<_>>();
                    let hull_min_x  = hull_x.iter().cloned().fold(f64::INFINITY, f64::min);
                    let hull_max_x  = hull_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                    if self.d_min > hull_max_x || self.d_max < hull_min_x {
                        // Convex hull is outside the line
                        None
                    } else {
                        // Convex hull is contained within the line
                        Some((0.0, 1.0))
                    }
                }
            } else {
                // Only clipped against the lower portion of the hull
                Some((t1, 1.0))
            }
        } else if t1 < 0.0 {
            // t2 may still be > 0.0 and form a valid line
            if t2 < 0.0 {
                None
            } else if t2 > 1.0 {
                Some((0.0, 1.0))
            } else {
                Some((0.0, t2))
            }
        } else if t1 > 1.0 {
            // t2 must be larger than t1 so no clip
            None
        } else {
            // Both in the range 0-1
            Some((t1, t2))
        }
    }
}

impl FatLine {
    ///
    /// Creates a new fatline from a central line and two points representing its outer edges
    /// 
    fn from_line_and_points<L: Line>(line: L, p1: L::Point, p2: L::Point) -> FatLine
    where L::Point: Coordinate+Coordinate2D {
        // Coefficients for the line
        let (a, b, c)   = line_coefficients_2d(&line);

        // Compute the distances to the control points
        let d1          = a*p1.x() + b*p1.y() + c;
        let d2          = a*p2.x() + b*p2.y() + c;

        // This is the 'estimated fit' shortcut suggested by Sederberg/Nishta in their paper rather than the tighest fitting line
        let (d_min, d_max) = if d1*d2 > 0.0 {
            // Both control points on the same side of the line
            (
                (3.0/4.0) * (d1.min(d2).min(0.0)),
                (3.0/4.0) * (d1.max(d2).max(0.0))
            )
        } else {
            // Control points on opposite sides of the line
            (
                (4.0/9.0) * (d1.min(d2).min(0.0)),
                (4.0/9.0) * (d1.max(d2).max(0.0))
            )
        };

        FatLine {
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }

    ///
    /// Creates a new fatline from a curve
    /// 
    pub fn from_curve<C: BezierCurve>(curve: &C) -> FatLine
    where C::Point: Coordinate+Coordinate2D {
        // Line between the start and end points of the curve
        let line        = (curve.start_point(), curve.end_point());
        let (cp1, cp2)  = curve.control_points();

        Self::from_line_and_points(line, cp1, cp2)
    }

    ///
    /// Creates a perpendicular fatline from a curve
    /// 
    pub fn from_curve_perpendicular<C: BezierCurve>(curve: &C) -> FatLine
    where C::Point: Coordinate+Coordinate2D {
        let (start_point, end_point) = (curve.start_point(), curve.end_point());

        // Line between the start and end points of the curve
        let line            = (start_point, end_point);

        // Mid-point of the line
        let mid_point       = line.point_at_pos(0.5);

        // Target point to generate a perpendicular line
        let offset          = mid_point - start_point;
        let offset          = C::Point::from_components(&[offset.y(), offset.x()]);
        let target_point    = mid_point + offset;

        // Perpendicular line
        let line            = (mid_point, target_point);

        // Compute the distances to all of the points
        let (cp1, cp2)      = curve.control_points();
        let (a, b, c)       = line_coefficients_2d(&line);

        let d1              = a*start_point.x() + b*start_point.y() + c;
        let d2              = a*cp1.x() + b*cp1.y() + c;
        let d3              = a*cp2.x() + b*cp2.y() + c;
        let d4              = a*end_point.x() + b*end_point.y() + c;

        // No approximation to improve the line fit here
        let d_min           = d1.min(d2).min(d3).min(d4);
        let d_max           = d1.max(d2).max(d3).max(d4);

        FatLine {
            coeff:  (a, b, c),
            d_min:  d_min,
            d_max:  d_max
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::super::bezier::*;

    impl FatLine {
        ///
        /// Creates a new fat line
        /// 
        pub fn new<L: Line>(line: L, d_min: f64, d_max: f64) -> FatLine
        where L::Point: Coordinate2D {
            let (a, b, c)   = line_coefficients_2d(&line);

            FatLine {
                d_min:  d_min,
                d_max:  d_max,
                coeff:  (a, b, c)
            }
        }

        ///
        /// Clips a bezier curve against this fat line. This returns a new bezier curve where
        /// parts that are outside the line have been clipped away, but does not necessarily
        /// guarantee that it's just the portion within the line.
        /// 
        /// This call can be iterated to improve the fit in many cases, and will return none
        /// in the case where the curve is not within the line.
        /// 
        pub fn clip<FromCurve: BezierCurve, ToCurve: BezierCurveFactory<Point=FromCurve::Point>>(&self, curve: &FromCurve) -> Option<ToCurve> 
        where FromCurve::Point: Coordinate2D {
            if let Some((t1, t2)) = self.clip_t(curve) {
                Some(ToCurve::from_curve(&curve.section(t1, t2)))
            } else {
                None
            }
        }
    }

    #[test]
    fn distance_to_horizontal_line() {
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);

        assert!((fat_line.distance(&Coord2(0.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(0.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(3.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(3.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(5.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(5.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(200.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(200.0, 0.0))- -4.0).abs() < 0.0001);
    }

    #[test]
    fn convex_hull_basic() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(5.0, 1.0/3.0), Coord2(6.0, 2.0/3.0));
        let hull        = FatLine::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==4);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(5.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(6.0, 2.0/3.0)) < 0.001);
        assert!(hull[3].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_concave_cp2() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(3.0, 2.0/3.0));
        let hull        = FatLine::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==3);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(4.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_concave_cp1() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(8.0, 2.0/3.0));
        let hull        = FatLine::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==3);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(8.0, 2.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_opposite_sides() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(1.0, 2.0/3.0));
        let hull        = FatLine::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==4);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(4.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
        assert!(hull[3].distance_to(&Coord2(1.0, 2.0/3.0)) < 0.001);
    }

    #[test]
    fn distance_curve_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line        = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve      = line_to_bezier::<_, Curve<_>>(&(Coord2(0.0, 0.0), Coord2(5.0, 8.0)));
        let distance_curve  = fat_line.distance_curve::<_, Curve<Coord2>>(&clip_curve);

        println!("{:?} {:?}", distance_curve.point_at_pos(0.0), distance_curve.point_at_pos(1.0));

        assert!((distance_curve.point_at_pos(0.0).x()- -4.0).abs() < 0.0001);
        assert!((distance_curve.point_at_pos(1.0).x()-4.0).abs() < 0.0001);
    }

    #[test]
    fn clip_line_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve  = line_to_bezier::<_, Curve<_>>(&(Coord2(0.0, 0.0), Coord2(5.0, 8.0)));

        let clipped     = fat_line.clip::<_, Curve<Coord2>>(&clip_curve).unwrap();
        let clipped     = fat_line.clip::<_, Curve<Coord2>>(&clipped).unwrap();
        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.0001);
        assert!((end_point.y()-7.0).abs() < 0.0001);
    }

    #[test]
    fn clip_t_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line        = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve      = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));
        let distance_curve  = fat_line.distance_curve::<_, Curve<Coord2>>(&clip_curve);

        let (t1, t2)    = fat_line.clip_t(&clip_curve).unwrap();
        let start_point = clip_curve.point_at_pos(t1);
        let end_point   = clip_curve.point_at_pos(t2);

        println!("Points on curve: {:?} {:?}", start_point, end_point);
        println!("Distance-x: {:?} {:?}", distance_curve.point_at_pos(t1).x(), distance_curve.point_at_pos(t2).x());
        println!("Distance-y: {:?} {:?}", distance_curve.point_at_pos(t1).y(), distance_curve.point_at_pos(t2).y());
        println!("T: {:?}", fat_line.clip_t(&clip_curve));

        assert!(start_point.y() <= 2.0);
        assert!(end_point.y() >= 7.0);
    }

    #[test]
    fn clip_curve_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let mut clipped = clip_curve.clone();

        // Should be able to iteratively refine to a curve clipped to the fat line
        for _x in 0..5 {
            let next_clipped = fat_line.clip(&clipped).unwrap();
            clipped = next_clipped;
        }

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.0001);
        assert!((end_point.y()-7.0).abs() < 0.0001);
    }

    #[test]
    fn clip_curve_in_line() {
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -16.0, 16.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let clipped = fat_line.clip::<_, Curve<Coord2>>(&clip_curve);
        assert!(clipped.is_some());
        let clipped = clipped.unwrap();

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.x()-0.0).abs() < 0.0001);
        assert!((end_point.x()-5.0).abs() < 0.0001);

        assert!((start_point.y()-0.0).abs() < 0.0001);
        assert!((end_point.y()-8.0).abs() < 0.0001);
    }

    #[test]
    fn clip_curve_start_in_line() {
        // If the start point is inside the fat line, we should only clip the end point
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -16.0, 3.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let clipped = fat_line.clip::<_, Curve<Coord2>>(&clip_curve);
        assert!(clipped.is_some());
        let clipped = clipped.unwrap();

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.x()-0.0).abs() < 0.0001);
        assert!(end_point.x() <= 5.0);

        assert!((start_point.y()-0.0).abs() < 0.0001);
        assert!(end_point.y() <= 8.0);
    }

    #[test]
    fn clip_curve_end_in_line() {
        // If the end point is inside the fat line, we should only clip the start point
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 16.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let clipped = fat_line.clip::<_, Curve<Coord2>>(&clip_curve);
        assert!(clipped.is_some());
        let clipped = clipped.unwrap();

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!(start_point.x() >= 0.0);
        assert!((end_point.x()-5.0).abs() < 0.0001);

        assert!(start_point.y() >= 0.0);
        assert!((end_point.y()-8.0).abs() < 0.0001);
    }

    #[test]
    fn clip_curve_outside_line() {
        // If the curve is entirely outside the line, we should return None
        let fat_line    = FatLine::new((Coord2(0.0, 20.0), Coord2(5.0, 20.0)), -2.0, 2.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let clipped = fat_line.clip::<_, Curve<Coord2>>(&clip_curve);
        assert!(clipped.is_none());
    }

    #[test]
    fn can_always_refine() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let mut clipped = clip_curve.clone();

        for _x in 0..100 {
            let next_clipped = fat_line.clip(&clipped);
            assert!(next_clipped.is_some());
            clipped = next_clipped.unwrap();
        }

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.0001);
        assert!((end_point.y()-7.0).abs() < 0.0001);
    }

    #[test]
    fn clip_curves_1() {
        // Two curves that clipped incorrectly from the clip intersection test
        let curve1 = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
        let curve2 = Curve::from_points(Coord2(67.25, 113.48), Coord2(181.38, 199.44), Coord2(146.18, 85.98), Coord2(109.35, 211.01));

        // Clip curve1 against curve2
        let fat_line = FatLine::from_curve(&curve2);
        let (t1, t2) = fat_line.clip_t(&curve1).unwrap();

        // Intersection points:
        //
        // Coord2(81.78, 109.88)
        // Coord2(133.16, 167.13)
        // Coord2(179.87, 199.67)

        println!("D_min: {:?}, D_max: {:?}", fat_line.d_min, fat_line.d_max);

        let distance = fat_line.distance_curve::<_, Curve<Coord2>>(&curve1);
        let hull = FatLine::distance_curve_convex_hull(&distance);
        println!("Distance convex hull: {:?}", hull);

        for t in 0..=10 {
            let t = (t as f64) / 10.0;

            let p1 = curve1.point_at_pos(t);
            let d1 = distance.point_at_pos(t);
            let d2 = fat_line.distance(&p1);

            println!("{} pos {:?}, dist {:?}, actual {:?}", t, p1, d1, d2);
        }
        println!("{:?} {:?}", (t1, t2), (curve1.point_at_pos(t2).x(), curve1.point_at_pos(t2).y()));

        assert!(curve1.point_at_pos(t1).x() < 81.79);
        assert!(curve1.point_at_pos(t2).x() > 179.86);
    }

    #[test]
    fn clip_curves_2() {
        // Intersection points:
        //
        // Coord2(81.78, 109.88)
        // Coord2(133.16, 167.13)
        // Coord2(179.87, 199.67)

        // Two curves that clipped incorrectly from the clip intersection test
        let curve1 = Curve::from_points(Coord2(67.25, 113.48), Coord2(210.0, 190.0), Coord2(155.03, 82.90), Coord2(99.65, 240.93));
        let curve2 = Curve::from_points(Coord2(77.5, 103.75), Coord2(220.0, 220.0), Coord2(97.5, 132.5), Coord2(130.0, 180.0));       

        // Clip curve1 against curve2
        let fat_line = FatLine::from_curve(&curve2);
        let (t1, t2) = fat_line.clip_t(&curve1).unwrap();

        println!("D_min: {:?}, D_max: {:?}", fat_line.d_min, fat_line.d_max);

        let distance = fat_line.distance_curve::<_, Curve<Coord2>>(&curve1);
        let hull = FatLine::distance_curve_convex_hull(&distance);
        println!("Distance convex hull: {:?}", hull);

        for t in 0..=10 {
            let t = (t as f64) / 10.0;

            let p1 = curve1.point_at_pos(t);
            let d1 = distance.point_at_pos(t);
            let d2 = fat_line.distance(&p1);

            println!("{} pos {:?}, dist {:?}, actual {:?}", t, p1, d1, d2);
        }
        println!("{:?} {:?}", (t1, t2), (curve1.point_at_pos(t2).x(), curve1.point_at_pos(t2).y()));

        assert!(curve1.point_at_pos(t1).x() < 81.79);
        assert!(curve1.point_at_pos(t2).x() > 179.86);
    }

    #[test]
    fn clip_curves_3() {
        // Intersection points:
        //
        // Coord2(81.78, 109.88)

        // Curve1 here forms a line that intercepts close to the start of Curve2, which seems to cause some accuracy issues

        let curve1 = Curve::from_points(Coord2(80.317, 107.796), Coord2(88.615, 119.383), Coord2(82.851, 111.424), Coord2(85.591, 115.301));
        let curve2 = Curve::from_points(Coord2(81.248, 109.971), Coord2(134.936, 171.219), Coord2(118.038, 104.934), Coord2(122.245, 142.970));       

        // Clip curve1 against curve2
        let fat_line = FatLine::from_curve(&curve2);
        let (t1, t2) = fat_line.clip_t(&curve1).unwrap();

        println!("D_min: {:?}, D_max: {:?}", fat_line.d_min, fat_line.d_max);

        let distance = fat_line.distance_curve::<_, Curve<Coord2>>(&curve1);
        let hull = FatLine::distance_curve_convex_hull(&distance);
        println!("Distance convex hull: {:?}", hull);

        for t in 0..=10 {
            let t = (t as f64) / 10.0;

            let p1 = curve1.point_at_pos(t);
            let d1 = distance.point_at_pos(t);
            let d2 = fat_line.distance(&p1);

            println!("{} pos {:?}, dist {:?}, actual {:?}", t, p1, d1, d2);
        }
        println!("{:?} {:?}", (t1, t2), (curve1.point_at_pos(t2).x(), curve1.point_at_pos(t2).y()));

        assert!(curve1.point_at_pos(t1).x() < 81.79);
        assert!(curve1.point_at_pos(t2).x() > 81.78);
    }

    #[test]
    fn clip_curves_4() {
        // Intersection points:
        //
        // Coord2(81.78, 109.88)

        let curve2 = Curve::from_points(Coord2(80.317, 107.796), Coord2(88.615, 119.383), Coord2(82.851, 111.424), Coord2(85.591, 115.301));
        let curve1 = Curve::from_points(Coord2(81.248, 109.971), Coord2(134.936, 171.219), Coord2(118.038, 104.934), Coord2(122.245, 142.970));       

        // Clip curve1 against curve2
        let fat_line = FatLine::from_curve(&curve2);
        let (t1, t2) = fat_line.clip_t(&curve1).unwrap();

        println!("D_min: {:?}, D_max: {:?}", fat_line.d_min, fat_line.d_max);

        let distance = fat_line.distance_curve::<_, Curve<Coord2>>(&curve1);
        let hull = FatLine::distance_curve_convex_hull(&distance);
        println!("Distance convex hull: {:?}", hull);

        for t in 0..=10 {
            let t = (t as f64) / 10.0;

            let p1 = curve1.point_at_pos(t);
            let d1 = distance.point_at_pos(t);
            let d2 = fat_line.distance(&p1);

            println!("{} pos {:?}, dist {:?}, actual {:?}", t, p1, d1, d2);
        }
        println!("{:?} {:?}", (t1, t2), (curve1.point_at_pos(t2).x(), curve1.point_at_pos(t2).y()));

        assert!(curve1.point_at_pos(t1).x() < 81.79);
        assert!(curve1.point_at_pos(t2).x() > 81.78);
    }
}
