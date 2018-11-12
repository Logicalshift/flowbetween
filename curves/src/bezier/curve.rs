use super::fit::*;
use super::basis::*;
use super::solve::*;
use super::search::*;
use super::bounds::*;
use super::subdivide::*;

use super::super::geo::*;
use super::super::coordinate::*;

const LENGTH_SUBDIVISIONS: usize = 16;

///
/// Trait implemented by bezier curves that can create new versions of themselves
/// 
pub trait BezierCurveFactory: BezierCurve {
    ///
    /// Creates a new bezier curve of the same type from some points
    /// 
    fn from_points(start: Self::Point, control_points: (Self::Point, Self::Point), end: Self::Point) -> Self;

    ///
    /// Creates a new bezier curve of this type from an equivalent curve of another type
    /// 
    #[inline]
    fn from_curve<Curve: BezierCurve<Point=Self::Point>>(curve: &Curve) -> Self {
        Self::from_points(curve.start_point(), curve.control_points(), curve.end_point())
    }

    ///
    /// Generates a curve by attempting to find a best fit against a set of points
    /// 
    #[inline]
    fn fit_from_points(points: &[Self::Point], max_error: f64) -> Option<Vec<Self>> {
        fit_curve(points, max_error)
    }
}

///
/// Trait implemented by things representing a cubic bezier curve
/// 
pub trait BezierCurve: Geo+Clone+Sized {
    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> Self::Point;

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> Self::Point;

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Self::Point, Self::Point);

    ///
    /// Reverses the direction of this curve
    /// 
    fn reverse<Curve: BezierCurveFactory<Point=Self::Point>>(self) -> Curve {
        let (cp1, cp2) = self.control_points();
        Curve::from_points(self.end_point(), (cp2, cp1), self.start_point())
    }

    ///
    /// Given a value t from 0 to 1, returns a point on this curve
    /// 
    #[inline]
    fn point_at_pos(&self, t: f64) -> Self::Point {
        let control_points = self.control_points();
        basis(t, self.start_point(), control_points.0, control_points.1, self.end_point())
    }

    ///
    /// Given a point that is on or very close to the curve, returns the t value where the point can be found
    /// (or None if the point is not very close to the curve)
    ///
    #[inline]
    fn t_for_point(&self, point: &Self::Point) -> Option<f64> {
        solve_curve_for_t(self, point)
    }

    ///
    /// Given a value t from 0 to 1, finds a point on this curve and subdivides it, returning the two resulting curves
    /// 
    #[inline]
    fn subdivide<Curve: BezierCurveFactory<Point=Self::Point>>(&self, t: f64) -> (Curve, Curve) {
        let control_points              = self.control_points();
        let (first_curve, second_curve) = subdivide4(t, self.start_point(), control_points.0, control_points.1, self.end_point());

        (Curve::from_points(first_curve.0, (first_curve.1, first_curve.2), first_curve.3),
            Curve::from_points(second_curve.0, (second_curve.1, second_curve.2), second_curve.3))
    }

    ///
    /// Computes the bounds of this bezier curve
    /// 
    fn bounding_box<Bounds: BoundingBox<Point=Self::Point>>(&self) -> Bounds {
        // Fetch the various points and the derivative of this curve
        let start       = self.start_point();
        let end         = self.end_point();
        let (cp1, cp2)  = self.control_points();

        bounding_box4(start, cp1, cp2, end)
    }
    
    ///
    /// Faster but less accurate bounding box for a curve
    /// 
    /// This will produce a bounding box that contains the curve but which may be larger than necessary
    /// 
    #[inline]
    fn fast_bounding_box<Bounds: BoundingBox<Point=Self::Point>>(&self) -> Bounds {
        let start           = self.start_point();
        let end             = self.end_point();
        let control_points  = self.control_points();

        let min             = Self::Point::from_smallest_components(start, end);
        let min             = Self::Point::from_smallest_components(min, control_points.0);
        let min             = Self::Point::from_smallest_components(min, control_points.1);

        let max             = Self::Point::from_biggest_components(start, end);
        let max             = Self::Point::from_biggest_components(max, control_points.0);
        let max             = Self::Point::from_biggest_components(max, control_points.1);

        Bounds::from_min_max(min, max)
    }

    ///
    /// Given a function that determines if a searched-for point is within a bounding box, searches the
    /// curve for the t values for the corresponding points
    /// 
    fn search_with_bounds<MatchFn: Fn(Self::Point, Self::Point) -> bool>(&self, max_error: f64, match_fn: MatchFn) -> Vec<f64> {
        // Fetch the various points and the derivative of this curve
        let start       = self.start_point();
        let end         = self.end_point();
        let (cp1, cp2)  = self.control_points();

        // Perform the search
        search_bounds4(max_error, start, cp1, cp2, end, match_fn)
    }

    ///
    /// Finds the t values where this curve has extremities
    /// 
    #[inline]
    fn find_extremities(&self) -> Vec<f64> {
        let start       = self.start_point();
        let end         = self.end_point();
        let (cp1, cp2)  = self.control_points();

        find_extremities(start, cp1, cp2, end)
    }

    ///
    /// Attempts to estimate the length of this curve
    /// 
    fn estimate_length(&self, max_t: f64) -> f64 {
        let mut last_pos = self.point_at_pos(0.0);
        let mut length   = 0.0;

        for t in 1..LENGTH_SUBDIVISIONS {
            let t           = (t as f64) / (LENGTH_SUBDIVISIONS as f64) * max_t;
            let next_pos    = self.point_at_pos(t);

            length += last_pos.distance_to(&next_pos);
            last_pos = next_pos;
        }

        length
    }
}

///
/// Represents a Bezier curve
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve<Coord: Coordinate> {
    pub start_point:    Coord,
    pub end_point:      Coord,
    pub control_points: (Coord, Coord)
}

impl<Coord: Coordinate> Geo for Curve<Coord> {
    type Point = Coord;
}

impl<Coord: Coordinate> BezierCurveFactory for Curve<Coord> {
    fn from_points(start: Coord, (control_point1, control_point2): (Coord, Coord), end: Coord) -> Self {
        Curve {
            start_point:    start,
            control_points: (control_point1, control_point2),
            end_point:      end
        }
    }
}

impl<Coord: Coordinate> BezierCurve for Curve<Coord> {
    #[inline]
    fn start_point(&self) -> Coord {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> Coord {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> (Coord, Coord) {
        self.control_points
    }
}
