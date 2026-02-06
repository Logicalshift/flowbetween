use super::shape::*;

use serde::*;

use flo_curves::geo::*;

use std::ops::{Add, DivAssign, Mul, MulAssign, Sub};

///
/// Represents a point on the canvas, 32-bit version used for serializing the data
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

impl Coordinate2D for CanvasPoint {
    ///
    /// X component of this coordinate
    ///
    #[inline]
    fn x(&self) -> f64 {
        self.x as f64
    }

    ///
    /// Y component of this coordinate
    ///
    #[inline]
    fn y(&self) -> f64 {
        self.y as f64
    }
}

impl Add for CanvasPoint {
    type Output = CanvasPoint;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        CanvasPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for CanvasPoint {
    type Output = CanvasPoint;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        CanvasPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for CanvasPoint {
    type Output = CanvasPoint;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        CanvasPoint {
            x: (self.x as f64 * rhs) as f32,
            y: (self.y as f64 * rhs) as f32,
        }
    }
}

impl MulAssign<f64> for CanvasPoint {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x = (self.x as f64 * rhs) as f32;
        self.y = (self.y as f64 * rhs) as f32;
    }
}

impl DivAssign<f64> for CanvasPoint {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.x = (self.x as f64 / rhs) as f32;
        self.y = (self.y as f64 / rhs) as f32;
    }
}

impl Coordinate for CanvasPoint {
    #[inline]
    fn from_components(components: &[f64]) -> CanvasPoint {
        CanvasPoint {
            x: components[0] as f32,
            y: components[1] as f32,
        }
    }

    #[inline]
    fn origin() -> CanvasPoint {
        CanvasPoint { x: 0.0, y: 0.0 }
    }

    #[inline]
    fn len() -> usize {
        2
    }

    #[inline]
    fn get(&self, index: usize) -> f64 {
        match index {
            0 => self.x as f64,
            1 => self.y as f64,
            _ => panic!("CanvasPoint only has two components"),
        }
    }

    fn from_biggest_components(p1: CanvasPoint, p2: CanvasPoint) -> CanvasPoint {
        CanvasPoint {
            x: p1.x.max(p2.x),
            y: p1.y.max(p2.y),
        }
    }

    fn from_smallest_components(p1: CanvasPoint, p2: CanvasPoint) -> CanvasPoint {
        CanvasPoint {
            x: p1.x.min(p2.x),
            y: p1.y.min(p2.y),
        }
    }

    #[inline]
    fn distance_to(&self, target: &CanvasPoint) -> f64 {
        let dist_x = target.x as f64 - self.x as f64;
        let dist_y = target.y as f64 - self.y as f64;

        f64::sqrt(dist_x * dist_x + dist_y * dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        (self.x as f64) * (target.x as f64) + (self.y as f64) * (target.y as f64)
    }
}

impl From<WorkingPoint> for CanvasPoint {
    fn from(point: WorkingPoint) -> Self {
        CanvasPoint {
            x: point.x as f32,
            y: point.y as f32,
        }
    }
}
