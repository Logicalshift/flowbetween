//!
//! Supplies the methods for taking a series of edit actions and applying
//! them to the objects to be edited.
//! 
//! Animation or layer implementations use this to commit edits.
//! 

mod animation_editor;
mod layer_editor;

pub use self::animation_editor::*;
pub use self::layer_editor::*;
