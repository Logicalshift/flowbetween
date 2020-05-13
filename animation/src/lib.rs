//!
//! Library for describing and editing FlowBetween animations
//!
#![warn(bare_trait_objects)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate modifier;
extern crate futures;
extern crate itertools;

mod traits;
mod onion_skin;
pub mod brushes;
pub mod raycast;
pub mod serializer;
pub mod storage;
pub mod editor;

pub use self::traits::*;
pub use self::onion_skin::*;
