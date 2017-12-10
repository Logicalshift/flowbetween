use std::ops::*;

///
/// Represents a value that can be used as a coordinate in a bezier curve
/// 
pub trait Coordinate : Sized+Copy+Add<Self, Output=Self>+Mul<f32, Output=Self>+Sub<Self, Output=Self> {
    /// The number of components in this coordinate
    fn len() -> usize;

    /// Retrieves the component at the specified index
    fn get(&self, index: usize) -> f32;

    /// Returns a point made up of the biggest components of the two points
    fn from_biggest_components(p1: Self, p2: Self) -> Self;

    /// Returns a point made up of the smallest components of the two points
    fn from_smallest_components(p1: Self, p2: Self) -> Self;
}

impl Coordinate for f32 {
    #[inline] fn len() -> usize { 1 }
    #[inline] fn get(&self, _index: usize) -> f32 { *self }

    #[inline]
    fn from_biggest_components(p1: f32, p2: f32) -> f32 {
        if p1 > p2 {
            p1
        } else {
            p2
        }
    }

    #[inline]
    fn from_smallest_components(p1: f32, p2: f32) -> f32 {
        if p1 < p2 {
            p1
        } else {
            p2
        }
    }
}

/// Represents a 2D point
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord2(pub f32, pub f32);

impl Coord2 {
    ///
    /// X component of this coordinate
    /// 
    #[inline]
    pub fn x(&self) -> f32 {
        self.0
    }

    ///
    /// Y component of this coordinate
    /// 
    #[inline]
    pub fn y(&self) -> f32 {
        self.1
    }
}

impl Add<Coord2> for Coord2 {
    type Output=Coord2;

    #[inline]
    fn add(self, rhs: Coord2) -> Coord2 {
        Coord2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Coord2> for Coord2 {
    type Output=Coord2;

    #[inline]
    fn sub(self, rhs: Coord2) -> Coord2 {
        Coord2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f32> for Coord2 {
    type Output=Coord2;

    #[inline]
    fn mul(self, rhs: f32) -> Coord2 {
        Coord2(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for Coord2 {
    #[inline]
    fn len() -> usize { 2 }

    #[inline]
    fn get(&self, index: usize) -> f32 { 
        match index {
            0 => self.0,
            1 => self.1,
            _ => panic!("Coord2 only has two components")
        }
    }

    fn from_biggest_components(p1: Coord2, p2: Coord2) -> Coord2 {
        Coord2(f32::from_biggest_components(p1.0, p2.0), f32::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: Coord2, p2: Coord2) -> Coord2 {
        Coord2(f32::from_smallest_components(p1.0, p2.0), f32::from_smallest_components(p1.1, p2.1))
    }
}
