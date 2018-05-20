//!
//! Supplies the methods for taking a series of edit actions and applying
//! them to the objects to be edited.
//! 
//! Animation or layer implementations use this to commit edits.
//! 

mod layer_editor;

pub use self::layer_editor::*;
