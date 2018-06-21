#![warn(bare_trait_objects)]

extern crate roots;
extern crate itertools;

pub mod bezier;
pub mod line;
pub mod arc;

pub mod coordinate;
pub use self::coordinate::*;

pub mod geo;
pub use self::geo::*;

pub use self::bezier::BezierCurve;
pub use self::line::Line;
