//!
//! # Manipulating and describing lines
//! 
//! While `flo_curves` deals mostly with curves, it also supplies a small library of functions for manipulating
//! lines. The `Line` trait can be implemented on other types that define lines, enabling them to be used anywhere
//! the library needs to perform an operation on a line.
//! 
//! The basic line type is simply a tuple of two points (that is, any tuple of two values of the same type that
//! implements `Coordinate`).
//!

mod line;
mod to_curve;
mod intersection;
mod coefficients;

pub use self::line::*;
pub use self::to_curve::*;
pub use self::coefficients::*;
pub use self::intersection::*;

pub use super::geo::*;
