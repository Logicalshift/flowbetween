//!
//! Tools in FlowBetween provide different ways of interacting with the canvas. They include obvious things
//! like, say, a brush or a selection tool, but FlowBetween generalises the concept so that things like the
//! colour and layer selection are also tools.
//!
//! Every tool has a group. Only one tool can be selected within a group, so groups might represent things
//! like the main tool, the colour, line properties, layer selections, etc. The idea here is that the
//! basic operation is very similar to how other 'canvas' type apps works where you pick a tool and its
//! properties, but tools from different groups can be joined together so the user can switch multiple
//! properties all at once.
//!
//! Tools can 'live' in multiple places. They typically start out in a tool dock, which is just a fixed
//! region on the left or right of the document that shows the icons for selected tools.
//!

pub (crate) mod tool_state;
pub (crate) mod tool_dock;

pub (crate) mod blobland;
pub (crate) mod physics_layer;
pub (crate) mod physics_object;
pub (crate) mod physics_tool;
pub (crate) mod physics_simulation;
pub (crate) mod physics_simulation_joints;

pub (crate) mod physics_simulation_object;

pub use tool_state::*;

#[cfg(test)] mod test;
