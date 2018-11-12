//!
//! # Routines for describing, querying and manipulating Bezier curves
//! 
//! Bezier curves are described by types that implement the `BezierCurve` trait, as a start point, an end point
//! and two control points. The `Curve` type is provided as a base implementation but as with the other traits,
//! the primary trait can be implemented on any suitable data structure. `BezierCurveFactory` is provided for
//! types that can create new instances of themselves.
//! 
//! Even for types that don't support the factory method, the `section()` method can be used to represent curve
//! subsections efficiently.
//! 
//! The `fit_curve()` function provides a way to fit a series of Bezier curves to one or more points using a
//! least-mean-squared algorithm.
//! 
//! The various `curve_intersects_X()` functions provide ways to determine where a curve meets another kind
//! of object.
//!

mod curve;
mod section;
mod basis;
mod subdivide;
mod derivative;
mod tangent;
mod normal;
mod bounds;
mod deform;
mod fit;
mod offset;
mod search;
mod solve;
mod overlaps;
mod intersection;

pub mod path;

pub use self::curve::*;
pub use self::section::*;
pub use self::basis::*;
pub use self::subdivide::*;
pub use self::derivative::*;
pub use self::tangent::*;
pub use self::normal::*;
pub use self::bounds::*;
pub use self::deform::*;
pub use self::fit::*;
pub use self::offset::*;
pub use self::search::*;
pub use self::solve::*;
pub use self::overlaps::*;
pub use self::intersection::*;

pub use super::geo::*;
