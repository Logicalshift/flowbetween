//!
//! # Gtk+ UI pipe for flo_ui
//! 
//! This provides a UI pipe that presents a user interface described by `flo_ui` using Gtk+ as the
//! front-end toolkit.
//! 

extern crate flo_ui;
extern crate flo_canvas;

extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate anymap;
extern crate futures;
extern crate itertools;

mod gtk_thread;
mod gtk_event;
mod gtk_action;
pub mod widgets;
mod session;

pub use self::gtk_thread::*;
pub use self::gtk_event::*;
pub use self::gtk_action::*;
pub use self::session::*;
