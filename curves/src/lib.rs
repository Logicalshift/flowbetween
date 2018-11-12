//!
//! ```toml
//! flo_curves = "0.2"
//! ```
//! 
//! flo_curves
//! ==========
//! 
//! `flo_curves` is a library providing routines for manipulating various types of curve, particularly Bezier curves. It provides
//! a grab-bag of algorithms, from the basis functions for generating points on a curve to collisions, fitting to points and even 
//! path arithmetic. It is built around traits, which makes it easy to use the provided algorithms with any data structure, though 
//! some defaults are provided.
//! 
//! `flo_curves` is designed as a support library for `flowbetween`, an animation tool I'm working on, but is also designed to work
//! stand-alone.
//!

#![warn(bare_trait_objects)]

extern crate roots;
extern crate itertools;

mod consts;
pub mod bezier;
pub mod line;
pub mod arc;

pub mod coordinate;
pub use self::coordinate::*;

pub mod geo;
pub use self::geo::*;

pub use self::bezier::BezierCurveFactory;
pub use self::bezier::BezierCurve;
pub use self::line::Line;
