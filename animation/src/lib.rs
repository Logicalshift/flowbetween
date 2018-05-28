//!
//! Library for describing and editing FlowBetween animations
//!
extern crate flo_curves as curves;
extern crate flo_canvas as canvas;
extern crate flo_float_encoder as float_encoder;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate modifier;
extern crate futures;

mod traits;
pub mod inmemory;
pub mod brushes;

mod deref_map;

pub use self::traits::*;
