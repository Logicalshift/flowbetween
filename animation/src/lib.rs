//!
//! Library for describing and editing FlowBetween animations
//!
extern crate curves;
extern crate canvas;
extern crate float_encoder;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate modifier;

mod traits;
pub mod editor;
pub mod inmemory;
pub mod brushes;

mod deref_map;

pub use self::traits::*;
