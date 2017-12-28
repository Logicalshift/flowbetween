//!
//! Library for describing and editing FlowBetween animations
//!

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate modifier;
extern crate curves;
extern crate ui;

mod traits;
pub mod inmemory;
pub mod brushes;

mod deref_map;

pub use self::traits::*;
