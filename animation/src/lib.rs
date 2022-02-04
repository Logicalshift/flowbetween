//!
//! Library for describing and editing FlowBetween animations
//!
//! FlowBetween communicates between its various components using a streaming interface. Animation
//! edits are send to an animation by calling the `edit()` function and then publishing `AnimationEdit`
//! requests. Reading back is accomplished using a more traditional interface.
//!
//! This is not the lowest tier of the storage structure: the animation uses an underlying storage structure
//! that the animation itself communicates with via `StorageCommand` requests. An in-memory version of the 
//! storage API is supplied for testing, and the `flo_sqlite_storage` library provides a more permanent version
//! for storing on-disk.
//!
//! The advantage of using streams to communicate between the larger components of FlowBetween is mostly that it
//! creates a low level of interdependency. It's also good for concurrency, makes it easy to monitor or change
//! the edits as they are made, useful for adding scripting, provides a means of debugging and handy for implementing
//! features like undo.
//!
#![warn(bare_trait_objects)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod traits;
mod onion_skin;
pub mod brushes;
pub mod raycast;
pub mod serializer;
pub mod storage;
pub mod editor;
pub mod undo;

pub use self::traits::*;
pub use self::onion_skin::*;
