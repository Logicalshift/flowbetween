//!
//! Library for describing and editing FlowBetween animations
//!
#![warn(bare_trait_objects)]

extern crate flo_curves;
extern crate flo_canvas;
extern crate flo_float_encoder;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate modifier;
extern crate futures;
extern crate itertools;

mod traits;
mod onionskin;
pub mod brushes;
pub mod raycast;

pub use self::traits::*;
pub use self::onionskin::*;
