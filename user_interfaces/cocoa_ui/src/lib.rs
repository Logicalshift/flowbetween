//!
//! Provides an implementation of the Cocoa UI described in the `flo_cocoa_pipe` crate.
//!

#[macro_use] extern crate objc;
#[macro_use] extern crate lazy_static;

mod app;
mod event;
mod session;
mod cocoa_ui;
mod property;
mod canvas_state;
mod canvas_context;
mod view_canvas;

#[allow(dead_code)]
mod core_graphics_ffi;

pub use self::app::*;
