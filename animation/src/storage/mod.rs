//!
//! The animation storage API is a stream of requests to a storage back-end, which
//! should in turn provide a stream of responses.
//!

mod storage_api;
mod editor;

pub use self::storage_api::*;
pub use self::editor::*;
