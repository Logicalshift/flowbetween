//!
//! # App scene
//!
//! The app scene manages the documents that are open in FlowBetween. This is more useful for operating systems
//! like OS X where the application can be open without any documents loaded.
//!

mod flowbetween;
mod document;

pub use flowbetween::*;
pub use document::*;
