use std::ops::*;

///
/// Represents a value that can be used as a coordinate in a bezier curve
/// 
pub trait Coordinate : Sized+Copy+Add<Self, Output=Self>+Mul<f32, Output=Self> {
}

impl Coordinate for f32 {
}

/// Represents a 2D point
#[derive(Copy, Clone)]
pub struct Coord2(pub f32, pub f32);

impl Add<Coord2> for Coord2 {
    type Output=Coord2;

    fn add(self, rhs: Coord2) -> Coord2 {
        Coord2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Mul<f32> for Coord2 {
    type Output=Coord2;

    fn mul(self, rhs: f32) -> Coord2 {
        Coord2(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for Coord2 {
}
