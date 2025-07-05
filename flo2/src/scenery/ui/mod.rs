//!
//! # UI
//!
//! This defines subprograms that can run parts of the UI for FlowBetween. These subprograms are likely re-usable in other projects.
//!

mod namespaces;
mod control;
mod control_id;
mod focus;
mod subprograms;
mod ui_path;
mod dialog_id;
mod dialog;
mod egui;
mod physics;
mod physics_tool;
mod physics_object;
mod binding_tracker;
mod colors;

pub use namespaces::*;
pub use control::*;
pub use control_id::*;
pub use focus::*;
pub use subprograms::*;
pub use ui_path::*;
pub use dialog_id::*;
pub use dialog::*;
pub use egui::*;
pub use physics::*;
pub use physics_tool::*;
pub use binding_tracker::*;
