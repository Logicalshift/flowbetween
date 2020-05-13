//!
//! The animation storage API is a stream of requests to a storage back-end, which
//! should in turn provide a stream of responses.
//! 
//! This is one of more involved examples of how FlowBetween decouples components. Different
//! components of FlowBetween communicate using streams. This avoids the need for techniques
//! such as dependency injection (components do not need to be aware of each other), and makes
//! it easier that more traditional architectures to test and re-use code from anywhere in the
//! application (everything high-level has a simple input and output stream and fewer direct
//! dependencies).
//! 
//! Most components have no feedback or weak feedback. The storage API is different in that
//! the storage layer must produce exactly one response to every command and must not get
//! out of sync. The UI layer works in a similar 'tight' feedback loop.
//! 
//! Implementing the storage API means it's possible to use the `create_editor()` call to 
//! generate an implementation of the `EditableAnimation` API. The storage layer is 
//! relatively simple
//!

pub (super) mod file_properties;
pub (super) mod layer_properties;
pub (super) mod storage_api;
pub (super) mod in_memory_storage;
pub (super) mod animation_loader;

#[cfg(test)] mod tests;

pub use super::editor::*;
pub use self::storage_api::*;
pub use self::in_memory_storage::*;
pub use self::animation_loader::*;
