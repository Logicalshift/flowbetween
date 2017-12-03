//!
//! Library for describing and editing FlowBetween animations
//!

extern crate modifier;
extern crate ui;

mod traits;
pub mod inmemory;
pub mod brushes;

mod deref_map;

pub use self::traits::*;
