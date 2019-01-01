//!
//! Provides an implementation of the Cocoa UI described in the `flo_cocoa_pipe` crate.
//!

#[macro_use] extern crate objc;
#[macro_use] extern crate lazy_static;

mod app;
mod session;

pub use self::app::*;
