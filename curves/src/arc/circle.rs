use super::super::coordinate::*;
use super::super::bezier::*;
use super::super::bezier::path::*;

use std::f64;

// TODO: would be nice to support generic coordinates here, though 2D circles
// are a lot simpler mathematically.

///
/// Represents a circle in 2 dimensions
/// 
#[derive(Clone, Copy)]
pub struct Circle<Coord: Coordinate2D+Coordinate> {
    /// The center of this circle
    pub center: Coord,

    /// The radius of this circle
    pub radius: f64
}

///
/// Represents an arc of a circle in 2 dimensions
/// 
#[derive(Clone, Copy)]
pub struct CircularArc<'a, Coord: 'a+Coordinate2D+Coordinate> {
    /// The circle that this is an arc of
    circle: &'a Circle<Coord>,

    /// The start point of this arc, in radians
    start_radians: f64,

    /// The end point of this arc, in radians
    end_radians: f64
}

impl<Coord: Coordinate2D+Coordinate> Circle<Coord> {
    ///
    /// Creates a new circle with a center and a radius
    /// 
    pub fn new(center: Coord, radius: f64) -> Circle<Coord> {
        Circle {
            center: center,
            radius: radius
        }
    }

    ///
    /// Returns an object representing an arc from this circle
    /// 
    pub fn arc<'a>(&'a self, start_radians: f64, end_radians: f64) -> CircularArc<'a, Coord> {
        CircularArc {
            circle:         self,
            start_radians:  start_radians,
            end_radians:    end_radians
        }
    }

    ///
    /// Returns a set of bezier curves that approximate this circle
    /// 
    pub fn to_curves<Curve: BezierCurveFactory<Point=Coord>>(&self) -> Vec<Curve> {
        // Angles to put the curves at (we need 4 curves for a decent approximation of a circle)
        let start_angle     = f64::consts::PI/4.0;
        let section_angle   = f64::consts::PI/2.0;
        let angles          = [
            start_angle, 
            start_angle + section_angle, 
            start_angle + section_angle*2.0,
            start_angle + section_angle*3.0];
        
        // Convert the angles into curves
        angles.into_iter()
            .map(|angle| self.arc(*angle, angle+section_angle).to_bezier_curve())
            .collect()
    }

    ///
    /// Returns a path that approximates this circle
    /// 
    pub fn to_path<P: BezierPathFactory<Point=Coord>>(&self) -> P {
        let curves = self.to_curves::<Curve<_>>();

        P::from_points(curves[0].start_point(), curves.into_iter().map(|curve| {
            let (cp1, cp2)  = curve.control_points();
            let end_point   = curve.end_point();

            (cp1, cp2, end_point)
        }))
    }
}

impl<'a, Coord: Coordinate2D+Coordinate> CircularArc<'a, Coord> {
    ///
    /// Converts this arc to a bezier curve
    /// 
    /// If this arc covers an angle > 90 degrees, the curve will
    /// be very inaccurate.
    /// 
    pub fn to_bezier_curve<Curve: BezierCurveFactory<Point=Coord>>(&self) -> Curve {
        // Algorithm described here: https://www.tinaja.com/glib/bezcirc2.pdf
        // Curve for the unit arc with its center at (1,0)
        let theta       = self.end_radians - self.start_radians;
        let (x0, y0)    = ((theta/2.0).cos(), (theta/2.0).sin());
        let (x1, y1)    = ((4.0-x0)/3.0, ((1.0-x0)*(3.0-x0)/(3.0*y0)));
        let (x2, y2)    = (x1, -y1);
        let (x3, y3)    = (x0, -y0);

        // Rotate so the curve starts at start_radians
        fn rotate(x: f64, y: f64, theta: f64) -> (f64, f64) {
            let (cos_theta, sin_theta) = (theta.cos(), theta.sin());

            (x*cos_theta + y*sin_theta, x*-sin_theta + y*cos_theta)
        }

        let angle = -(f64::consts::PI/2.0-(theta/2.0));
        let angle = angle + self.start_radians;

        let (x0, y0) = rotate(x0, y0, angle);
        let (x1, y1) = rotate(x1, y1, angle);
        let (x2, y2) = rotate(x2, y2, angle);
        let (x3, y3) = rotate(x3, y3, angle);

        // Scale by radius
        let radius = self.circle.radius;
        let (x0, y0) = (x0*radius, y0*radius);
        let (x1, y1) = (x1*radius, y1*radius);
        let (x2, y2) = (x2*radius, y2*radius);
        let (x3, y3) = (x3*radius, y3*radius);

        // Translate by center
        let center = &self.circle.center;
        let (x0, y0) = (x0+center.x(), y0+center.y());
        let (x1, y1) = (x1+center.x(), y1+center.y());
        let (x2, y2) = (x2+center.x(), y2+center.y());
        let (x3, y3) = (x3+center.x(), y3+center.y());

        // Create the curve
        let p0 = Coord::from_components(&[x0, y0]);
        let p1 = Coord::from_components(&[x1, y1]);
        let p2 = Coord::from_components(&[x2, y2]);
        let p3 = Coord::from_components(&[x3, y3]);

        Curve::from_points(p0, p3, p1, p2)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64;

    #[test]
    fn can_convert_unit_arc() {
        let circle          = Circle::new(Coord2(0.0, 0.0), 1.0);
        let arc             = circle.arc(0.0, f64::consts::PI/2.0);
        let curve: Curve<_> = arc.to_bezier_curve();

        assert!(curve.start_point().distance_to(&Coord2(0.0, 1.0)) < 0.01);
        assert!(curve.end_point().distance_to(&Coord2(1.0, 0.0)) < 0.01);
    }

    #[test]
    fn circle_is_roughly_circular() {
        let circle = Circle::new(Coord2(0.0, 0.0), 1.0);

        for curve in circle.to_curves::<Curve<_>>() {
            for t in 0..=10 {
                let t = (t as f64)/10.0;
                let p = curve.point_at_pos(t);
                assert!((p.distance_to(&Coord2(0.0, 0.0))-1.0).abs() < 0.01);
            }
        }
    }

    #[test]
    fn circle_path_is_roughly_circular() {
        let circle = Circle::new(Coord2(5.0, 5.0), 4.0);

        for curve in path_to_curves::<_, Curve<_>>(&circle.to_path::<SimpleBezierPath>()) {
            for t in 0..=10 {
                let t = (t as f64)/10.0;
                let p = curve.point_at_pos(t);
                assert!((p.distance_to(&Coord2(5.0, 5.0))-4.0).abs() < 0.01);
            }
        }
    }
}