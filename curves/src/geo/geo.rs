use super::super::coordinate::*;

///
/// Simple base trait implemented by things representing geometry
/// 
pub trait Geo {
    /// The type of a point in this geometry
    type Point: Coordinate;
}
