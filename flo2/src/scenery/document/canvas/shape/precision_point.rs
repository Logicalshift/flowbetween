use super::super::point::*;

use serde::*;

use flo_curves::geo::*;

use std::ops::{Add, DivAssign, Mul, MulAssign, Sub};

///
/// Represents a point on the canvas, 64-bit version used for performing path operations that require precision (like path boolean operations)
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct CanvasPrecisionPoint {
    pub x: f64,
    pub y: f64,
}

impl Coordinate2D for CanvasPrecisionPoint {
    ///
    /// X component of this coordinate
    ///
    #[inline]
    fn x(&self) -> f64 {
        self.x
    }

    ///
    /// Y component of this coordinate
    ///
    #[inline]
    fn y(&self) -> f64 {
        self.y
    }
}

impl Add for CanvasPrecisionPoint {
    type Output = CanvasPrecisionPoint;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        CanvasPrecisionPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for CanvasPrecisionPoint {
    type Output = CanvasPrecisionPoint;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        CanvasPrecisionPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for CanvasPrecisionPoint {
    type Output = CanvasPrecisionPoint;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        CanvasPrecisionPoint {
            x: (self.x * rhs),
            y: (self.y  * rhs),
        }
    }
}

impl MulAssign<f64> for CanvasPrecisionPoint {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl DivAssign<f64> for CanvasPrecisionPoint {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl Coordinate for CanvasPrecisionPoint {
    #[inline]
    fn from_components(components: &[f64]) -> CanvasPrecisionPoint {
        CanvasPrecisionPoint {
            x: components[0],
            y: components[1],
        }
    }

    #[inline]
    fn origin() -> CanvasPrecisionPoint {
        CanvasPrecisionPoint { x: 0.0, y: 0.0 }
    }

    #[inline]
    fn len() -> usize {
        2
    }

    #[inline]
    fn get(&self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("CanvasPrecisionPoint only has two components"),
        }
    }

    fn from_biggest_components(p1: CanvasPrecisionPoint, p2: CanvasPrecisionPoint) -> CanvasPrecisionPoint {
        CanvasPrecisionPoint {
            x: p1.x.max(p2.x),
            y: p1.y.max(p2.y),
        }
    }

    fn from_smallest_components(p1: CanvasPrecisionPoint, p2: CanvasPrecisionPoint) -> CanvasPrecisionPoint {
        CanvasPrecisionPoint {
            x: p1.x.min(p2.x),
            y: p1.y.min(p2.y),
        }
    }

    #[inline]
    fn distance_to(&self, target: &CanvasPrecisionPoint) -> f64 {
        let dist_x = target.x - self.x;
        let dist_y = target.y - self.y;

        f64::sqrt(dist_x * dist_x + dist_y * dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        (self.x) * (target.x) + (self.y) * (target.y)
    }
}

impl From<CanvasPoint> for CanvasPrecisionPoint {
    fn from(point: CanvasPoint) -> Self {
        CanvasPrecisionPoint {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}
