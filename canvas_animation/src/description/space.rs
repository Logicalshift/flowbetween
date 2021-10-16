use serde::{Serialize, Deserialize};

use flo_curves::{Coordinate, Coordinate2D, Geo};
use flo_curves::bezier;
use flo_canvas::*;

use std::f64;
use std::ops::*;

// TODO: fix naming clash between BezierPath the structure and BezierPath the trait
// TODO: some of these types (in particular scale, rotate, etc) probably belong in flo_curves

///
/// A point in 2D space
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point2D(pub f64, pub f64);

///
/// Represents a scale factor
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale(pub f64, pub f64);

///
/// A rotation measured in degrees
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RotateDegrees(pub f64);

///
/// A rotation measured in radians
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RotateRadians(pub f64);

///
/// A transformed point. The parameters represent the offset and how the point is scaled and rotated
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformPoint(pub Point2D, pub Scale, pub RotateRadians);

///
/// A transformed point with an anchor point
///
/// (The anchor point is considered the origin for the purposes of the transformation)
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformWithAnchor(pub Point2D, pub TransformPoint);

///
/// Two control points followed by an end point (a point on a bezier curve)
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierPoint(pub Point2D, pub Point2D, pub Point2D);

///
/// A path made up of bezier curves
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierPath(pub Point2D, pub Vec<BezierPoint>);

impl Default for Point2D {
    fn default() -> Self {
        Point2D(0.0, 0.0)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale(1.0, 1.0)
    }
}

impl Default for RotateDegrees {
    fn default() -> Self {
        RotateDegrees(0.0)
    }
}

impl Default for RotateRadians {
    fn default() -> Self {
        RotateRadians(0.0)
    }
}

impl Into<RotateDegrees> for RotateRadians {
    fn into(self) -> RotateDegrees {
        let RotateRadians(radians) = self;
        RotateDegrees((radians / f64::consts::PI) * 180.0)
    }
}

impl Into<RotateRadians> for RotateDegrees {
    fn into(self) -> RotateRadians {
        let RotateDegrees(degrees) = self;
        RotateRadians((degrees / 180.0) * f64::consts::PI)
    }
}

impl Into<Transform2D> for RotateRadians {
    fn into(self) -> Transform2D {
        let RotateRadians(radians) = self;
        Transform2D::rotate(radians as _)
    }
}

impl Into<Transform2D> for RotateDegrees {
    fn into(self) -> Transform2D {
        let RotateDegrees(degrees) = self;
        Transform2D::rotate_degrees(degrees as _)
    }
}

impl Into<Transform2D> for Scale {
    fn into(self) -> Transform2D {
        let Scale(scale_x, scale_y) = self;
        Transform2D::scale(scale_x as _, scale_y as _)
    }
}

impl Into<Transform2D> for TransformPoint {
    fn into(self) -> Transform2D {
        let TransformPoint(Point2D(dx, dy), scale, rotate) = self;

        let translate   = Transform2D::translate(dx as _, dy as _);
        let scale       = scale.into();
        let rotate      = rotate.into();

        translate * rotate * scale
    }
}

impl Into<Transform2D> for TransformWithAnchor {
    fn into(self) -> Transform2D {
        let TransformWithAnchor(Point2D(anchor_x, anchor_y), transform) = self;

        let move_to_anchor  = Transform2D::translate(-anchor_x as _, -anchor_y as _);
        let transform       = transform.into();
        let move_back       = Transform2D::translate(anchor_x as _, anchor_y as _);

        move_back * transform * move_to_anchor
    }
}

impl Point2D {
    #[inline] pub fn x(&self) -> f64 { self.0 }
    #[inline] pub fn y(&self) -> f64 { self.1 }

    ///
    /// Appies a 2D transformation to this point
    ///
    pub fn transform(&self, transform: &Transform2D) -> Point2D {
        let Point2D(x, y)   = self;
        let (x, y)          = transform.transform_point(*x as _, *y as _);

        Point2D(x as _, y as _)
    }
}

impl Add<Point2D> for Point2D {
    type Output=Point2D;

    #[inline]
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Point2D> for Point2D {
    type Output=Point2D;

    #[inline]
    fn sub(self, rhs: Point2D) -> Point2D {
        Point2D(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f64> for Point2D {
    type Output=Point2D;

    #[inline]
    fn mul(self, rhs: f64) -> Point2D {
        Point2D(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for Point2D {
    #[inline]
    fn from_components(components: &[f64]) -> Point2D {
        Point2D(components[0], components[1])
    }

    #[inline]
    fn origin() -> Point2D {
        Point2D(0.0, 0.0)
    }

    #[inline]
    fn len() -> usize { 2 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0,
            1 => self.1,
            _ => panic!("Point2D only has two components")
        }
    }

    fn from_biggest_components(p1: Point2D, p2: Point2D) -> Point2D {
        Point2D(f64::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: Point2D, p2: Point2D) -> Point2D {
        Point2D(f64::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1))
    }

    #[inline]
    fn distance_to(&self, target: &Point2D) -> f64 {
        let dist_x = target.0-self.0;
        let dist_y = target.1-self.1;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.0*target.0 + self.1*target.1
    }
}

impl Coordinate2D for Point2D {
    #[inline]
    fn x(&self) -> f64 {
        self.0
    }

    #[inline]
    fn y(&self) -> f64 {
        self.1
    }
}

impl Add<TransformPoint> for TransformPoint {
    type Output=TransformPoint;

    #[inline]
    fn add(self, rhs: TransformPoint) -> TransformPoint {
        let TransformPoint(l_translate, Scale(l_scale_x, l_scale_y), RotateRadians(l_rotation)) = self;
        let TransformPoint(r_translate, Scale(r_scale_x, r_scale_y), RotateRadians(r_rotation)) = rhs;

        TransformPoint(l_translate + r_translate, Scale(l_scale_x + r_scale_x, l_scale_y + r_scale_y), RotateRadians(l_rotation + r_rotation))
    }
}

impl Sub<TransformPoint> for TransformPoint {
    type Output=TransformPoint;

    #[inline]
    fn sub(self, rhs: TransformPoint) -> TransformPoint {
        let TransformPoint(l_translate, Scale(l_scale_x, l_scale_y), RotateRadians(l_rotation)) = self;
        let TransformPoint(r_translate, Scale(r_scale_x, r_scale_y), RotateRadians(r_rotation)) = rhs;

        TransformPoint(l_translate - r_translate, Scale(l_scale_x - r_scale_x, l_scale_y - r_scale_y), RotateRadians(l_rotation - r_rotation))
    }
}

impl Mul<f64> for TransformPoint {
    type Output=TransformPoint;

    #[inline]
    fn mul(self, rhs: f64) -> TransformPoint {
        let TransformPoint(translate, Scale(scale_x, scale_y), RotateRadians(rotation)) = self;

        TransformPoint(translate * rhs, Scale(scale_x * rhs, scale_y * rhs), RotateRadians(rotation * rhs))
    }
}

impl Coordinate for TransformPoint {
    #[inline]
    fn from_components(components: &[f64]) -> TransformPoint {
        TransformPoint(Point2D(components[0], components[1]), Scale(components[2], components[3]), RotateRadians(components[4]))
    }

    #[inline]
    fn origin() -> TransformPoint {
        TransformPoint(Point2D::default(), Scale::default(), RotateRadians::default())
    }

    #[inline]
    fn len() -> usize { 5 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        let TransformPoint(Point2D(x, y), Scale(scale_x, scale_y), RotateRadians(rotation)) = self;

        match index {
            0 => *x,
            1 => *y,
            2 => *scale_x,
            3 => *scale_y,
            4 => *rotation,
            _ => panic!("TransformPoint only has five components")
        }
    }

    fn from_biggest_components(p1: TransformPoint, p2: TransformPoint) -> TransformPoint {
        let TransformPoint(Point2D(p1x, p1y), Scale(p1scale_x, p1scale_y), RotateRadians(p1rotation)) = p1;
        let TransformPoint(Point2D(p2x, p2y), Scale(p2scale_x, p2scale_y), RotateRadians(p2rotation)) = p2;

        TransformPoint(
            Point2D(f64::from_biggest_components(p1x, p2x), f64::from_biggest_components(p1y, p2y)),
            Scale(f64::from_biggest_components(p1scale_x, p2scale_x), f64::from_biggest_components(p1scale_y, p2scale_y)),
            RotateRadians(f64::from_biggest_components(p1rotation, p2rotation))
        )
    }

    fn from_smallest_components(p1: TransformPoint, p2: TransformPoint) -> TransformPoint {
        let TransformPoint(Point2D(p1x, p1y), Scale(p1scale_x, p1scale_y), RotateRadians(p1rotation)) = p1;
        let TransformPoint(Point2D(p2x, p2y), Scale(p2scale_x, p2scale_y), RotateRadians(p2rotation)) = p2;

        TransformPoint(
            Point2D(f64::from_smallest_components(p1x, p2x), f64::from_smallest_components(p1y, p2y)),
            Scale(f64::from_smallest_components(p1scale_x, p2scale_x), f64::from_smallest_components(p1scale_y, p2scale_y)),
            RotateRadians(f64::from_smallest_components(p1rotation, p2rotation))
        )
    }
}

impl Geo for BezierPath {
    type Point = Point2D;
}

impl bezier::path::BezierPath for BezierPath {
    type PointIter = Box<dyn Iterator<Item=(Self::Point, Self::Point, Self::Point)>>;

    fn start_point(&self) -> Self::Point {
        self.0
    }

    fn points(&self) -> Self::PointIter {
        Box::new(self.1.clone()
            .into_iter()
            .map(|BezierPoint(cp1, cp2, ep)| (cp1, cp2, ep)))
    }
}

impl bezier::path::BezierPathFactory for BezierPath {
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self {
        BezierPath(start_point, points.into_iter().map(|(cp1, cp2, ep)| BezierPoint(cp1, cp2, ep)).collect())
    }
}
