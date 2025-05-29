//!
//! # Document scene
//!
//! The document module contains subprograms relating to individual documents in FlowBetween. Each document runs
//! in its own scene, which is generally shut down at the point where all of the windows for that document are 
//! closed.
//!

mod document_id;
mod subprograms;
mod flowbetween_document;

pub use document_id::*;
pub use subprograms::*;
pub use flowbetween_document::*;
