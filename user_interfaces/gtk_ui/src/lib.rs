//!
//! # Gtk+ UI pipe for flo_ui
//!
//! This provides a UI pipe that presents a user interface described by `flo_ui` using Gtk+ as the
//! front-end toolkit.
//!
#![warn(bare_trait_objects)]

extern crate flo_ui;
extern crate flo_canvas;

extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gio;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate glib;
extern crate anymap;
extern crate futures;
extern crate itertools;
extern crate gl;
extern crate epoxy;
extern crate shared_library;
extern crate time;

#[macro_use]
extern crate lazy_static;

mod gtk_thread;
mod gtk_event;
mod gtk_event_parameter;
mod gtk_widget_event_type;
mod gtk_action;
pub mod widgets;
pub mod canvas;
mod session;

pub use self::gtk_thread::*;
pub use self::gtk_event::*;
pub use self::gtk_event_parameter::*;
pub use self::gtk_widget_event_type::*;
pub use self::gtk_action::*;
pub use self::session::*;
