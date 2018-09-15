use super::super::curve::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

///
/// A 'fat line' is a line with a width. It's used in bezier intersection algorithms,
/// in particular the clipping algorithm described by Sederberg and Nishita
/// 
pub struct FatLine<Line> {
    /// The thin line L'
    line: Line,

    /// The distance from the line to the upper part of the 'fat line'
    d_min: f64,

    /// The distance from the line to the lower part of the 'fat line'
    d_max: f64,

    /// The coefficients (a, b, c) in the equation ax+bx+c (where a^2+b^2 = 0)
    coeff: (f64, f64, f64)
}

impl<L: Line> FatLine<L>
where L::Point: Coordinate2D {
    ///
    /// Creates a new fat line
    /// 
    pub fn new(line: L, d_min: f64, d_max: f64) -> FatLine<L> {
        let (from, to)  = line.points();
        let (a, b, c)   = line_coefficients_2d(&line);

        FatLine {
            line:   line,
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }

    ///
    /// Returns the distance between the point and the central line
    /// 
    #[inline]
    pub fn distance(&self, point: &L::Point) -> f64 {
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
    pub fn distance_curve<C: BezierCurveFactory<Point=L::Point>>(&self, curve: &C) -> C {
        let (cp1, cp2)  = curve.control_points();

        let start       = L::Point::from_components(&[self.distance(&curve.start_point()), 0.0]);
        let end         = L::Point::from_components(&[self.distance(&curve.end_point()), 1.0]);
        let cp1         = L::Point::from_components(&[self.distance(&cp1), 1.0/3.0]);
        let cp2         = L::Point::from_components(&[self.distance(&cp2), 2.0/3.0]);

        C::from_points(start, end, cp1, cp2)
    }

    ///
    /// Returns the convex hull of a curve returned by distance_curve
    /// 
    /// We can use some of the properties of the distance_curve to simplify how this
    /// is worked out (specifically, we know the points are sorted vertically already
    /// so we only need to know if the two control points are on the same side or not)
    /// 
    fn distance_curve_convex_hull<C: BezierCurve<Point=L::Point>>(&self, distance_curve: &C) -> Vec<L::Point> {
        // Read the points from the curve
        let start       = curve.start_point();
        let (cp1, cp2)  = curve.control_points();
        let end         = curve.end_point();

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
}

impl<P: Coordinate+Coordinate2D> FatLine<(P, P)> {
    ///
    /// Creates a new fatline from a curve
    /// 
    pub fn from_curve<C: BezierCurve<Point=P>>(curve: &C) -> FatLine<(P, P)> {
        // Line between the start and end points of the curve
        let line        = (curve.start_point(), curve.end_point());
        
        // Coefficients for the line
        let (a, b, c)   = line_coefficients_2d(&line);

        // Compute the distances to the control points
        let (cp1, cp2)  = curve.control_points();
        let d1          = a*cp1.x() + b*cp1.y() + c;
        let d2          = a*cp2.x() + b*cp2.y() + c;

        // This is the 'estimated fit' shortcut suggested by Sederberg/Nishta in their paper rather than the tighest ffitting line
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
            line:   line,
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }
}