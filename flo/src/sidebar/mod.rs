//!
//! The sidebar is used for editing operations that won't fit into the toolbar or which deal with viewing or changing properties
//! on the selection.
//!

pub mod panel;
mod document_settings;
mod sidebar_controller;

pub use self::panel::*;
pub use self::document_settings::*;
pub use self::sidebar_controller::*;
