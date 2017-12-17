use std::ops::*;

///
/// Represents a value that can be used as a coordinate in a bezier curve
/// 
pub trait Coordinate : Sized+Copy+Add<Self, Output=Self>+Mul<f32, Output=Self>+Sub<Self, Output=Self> {
    /// Returns the origin coordinate
    fn origin() -> Self;

    /// The number of components in this coordinate
    fn len() -> usize;

    /// Retrieves the component at the specified index
    fn get(&self, index: usize) -> f32;

    /// Returns a point made up of the biggest components of the two points
    fn from_biggest_components(p1: Self, p2: Self) -> Self;

    /// Returns a point made up of the smallest components of the two points
    fn from_smallest_components(p1: Self, p2: Self) -> Self;

    /// Computes the distance between this coordinate and another of the same type
    #[inline]
    fn distance_to(&self, target: &Self) -> f32 {
        let mut squared_distance = 0.0;

        for component_index in 0..Self::len() {
            let component_distance = target.get(component_index) - self.get(component_index);
            squared_distance += component_distance * component_distance;
        }

        f32::sqrt(squared_distance)
    }

    /// Computes the dot product for this vector along with another vector
    #[inline]
    fn dot(&self, target: &Self) -> f32 {
        let mut dot_product = 0.0;

        for component_index in 0..Self::len() {
            dot_product += self.get(component_index) * target.get(component_index);
        }

        dot_product
    }

    /// Computes the magnitude of this vector
    #[inline]
    fn magnitude(&self) -> f32 {
        self.distance_to(&Self::origin())
    }

    /// Treating this as a vector, returns a unit vector in the same direction
    #[inline]
    fn normalize(&self) -> Self {
        *self * (1.0/self.magnitude())
    }
}

///
/// Represents a coordinate with a 2D position
/// 
pub trait Coordinate2D {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

impl Coordinate for f32 {
    #[inline] fn origin() -> f32 { 0.0 }
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

    #[inline]
    fn distance_to(&self, target: &f32) -> f32 {
        f32::abs(self-target)
    }

    fn dot(&self, target: &f32) -> f32 {
        self * target
    }
}

/// Represents a 2D point
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord2(pub f32, pub f32);

impl Coordinate2D for Coord2 {
    ///
    /// X component of this coordinate
    /// 
    #[inline]
    fn x(&self) -> f32 {
        self.0
    }

    ///
    /// Y component of this coordinate
    /// 
    #[inline]
    fn y(&self) -> f32 {
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
    fn origin() -> Coord2 {
        Coord2(0.0, 0.0)
    }

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

    #[inline]
    fn distance_to(&self, target: &Coord2) -> f32 {
        let dist_x = target.0-self.0;
        let dist_y = target.1-self.1;

        f32::sqrt(dist_x*dist_x + dist_y*dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f32 {
        self.0*target.0 + self.1*target.1
    }
}
